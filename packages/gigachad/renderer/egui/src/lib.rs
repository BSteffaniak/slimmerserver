#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, LazyLock, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use async_trait::async_trait;
use canvas::CanvasAction;
use eframe::egui::{self, Color32, CursorIcon, Response, Ui, Widget};
use flume::{Receiver, Sender};
use gigachad_actions::{ActionTrigger, ActionType, ElementTarget, StyleAction};
use gigachad_renderer::canvas::CanvasUpdate;
use gigachad_renderer::viewport::immediate::{Pos, Viewport, ViewportListener};
pub use gigachad_renderer::*;
use gigachad_router::Router;
use gigachad_transformer::{
    calc::Calc,
    models::{
        AlignItems, Cursor, JustifyContent, LayoutDirection, LayoutOverflow, LayoutPosition,
        Position, Route, Visibility,
    },
    ContainerElement, Element, Input, TableIter,
};
use itertools::Itertools;

#[derive(Clone)]
pub struct EguiRenderer {
    width: Option<u16>,
    height: Option<u16>,
    x: Option<i32>,
    y: Option<i32>,
    app: EguiApp,
    receiver: Receiver<String>,
}

impl EguiRenderer {
    #[must_use]
    pub fn new(
        router: Router,
        request_action: Sender<String>,
        on_resize: Sender<(f32, f32)>,
    ) -> Self {
        let (tx, rx) = flume::unbounded();
        let (event_tx, event_rx) = flume::unbounded();
        Self {
            width: None,
            height: None,
            x: None,
            y: None,
            app: EguiApp::new(router, tx, event_tx, event_rx, request_action, on_resize),
            receiver: rx,
        }
    }

    #[must_use]
    pub async fn wait_for_navigation(&self) -> Option<String> {
        self.receiver.recv_async().await.ok()
    }
}

pub struct EguiRenderRunner {
    width: u16,
    height: u16,
    x: Option<i32>,
    y: Option<i32>,
    app: EguiApp,
}

impl RenderRunner for EguiRenderRunner {
    /// # Errors
    ///
    /// Will error if egui fails to run the event loop.
    fn run(&mut self) -> Result<(), Box<dyn std::error::Error + Send>> {
        let mut viewport = egui::ViewportBuilder::default()
            .with_inner_size([f32::from(self.width), f32::from(self.height)]);

        #[allow(clippy::cast_precision_loss)]
        if let (Some(x), Some(y)) = (self.x, self.y) {
            viewport = viewport.with_position((x as f32, y as f32));
        }

        let options = eframe::NativeOptions {
            viewport,
            centered: true,
            #[cfg(feature = "wgpu")]
            renderer: eframe::Renderer::Wgpu,
            ..Default::default()
        };

        log::debug!("run: starting");
        if let Err(e) = eframe::run_native(
            "MoosicBox",
            options,
            Box::new(|cc| {
                egui_extras::install_image_loaders(&cc.egui_ctx);
                let app = self.app.clone();
                *app.ctx.write().unwrap() = Some(cc.egui_ctx.clone());
                Ok(Box::new(app))
            }),
        ) {
            log::error!("run: eframe error: {e:?}");
        }
        log::debug!("run: finished");

        Ok(())
    }
}

#[async_trait]
impl Renderer for EguiRenderer {
    /// # Panics
    ///
    /// Will panic if elements `Mutex` is poisoned.
    ///
    /// # Errors
    ///
    /// Will error if egui app fails to start
    async fn init(
        &mut self,
        width: u16,
        height: u16,
        x: Option<i32>,
        y: Option<i32>,
        background: Option<Color>,
    ) -> Result<(), Box<dyn std::error::Error + Send + 'static>> {
        self.width = Some(width);
        self.height = Some(height);
        self.x = x;
        self.y = y;
        self.app.background = background.map(Into::into);

        log::debug!("start: spawning listen thread");
        moosicbox_task::spawn("renderer_egui::start: listen", {
            let app = self.app.clone();
            async move {
                log::debug!("start: listening");
                app.listen().await;
                Ok::<_, Box<dyn std::error::Error + Send + 'static>>(())
            }
        });

        Ok(())
    }

    /// # Errors
    ///
    /// Will error if egui fails to run the event loop.
    async fn to_runner(
        &mut self,
    ) -> Result<Box<dyn RenderRunner>, Box<dyn std::error::Error + Send>> {
        Ok(Box::new(EguiRenderRunner {
            width: self.width.unwrap(),
            height: self.height.unwrap(),
            x: self.x,
            y: self.y,
            app: self.app.clone(),
        }))
    }

    /// # Errors
    ///
    /// Will error if egui fails to render the view.
    ///
    /// # Panics
    ///
    /// Will panic if elements `Mutex` is poisoned.
    fn render(&mut self, view: View) -> Result<(), Box<dyn std::error::Error + Send + 'static>> {
        moosicbox_logging::debug_or_trace!(
            ("render: start"),
            ("render: start {:?}", view.immediate)
        );
        let mut elements = view.immediate;

        elements.calculated_width = self.app.width.read().unwrap().or(self.width.map(f32::from));
        elements.calculated_height = self
            .app
            .height
            .read()
            .unwrap()
            .or(self.height.map(f32::from));
        elements.calc();
        *self.app.container.write().unwrap() = elements;
        self.app.images.write().unwrap().clear();
        self.app.viewport_listeners.write().unwrap().clear();
        self.app.route_requests.write().unwrap().clear();
        self.app.checkboxes.write().unwrap().clear();

        log::debug!("render: finished");
        if let Some(ctx) = &*self.app.ctx.read().unwrap() {
            ctx.request_repaint();
        }

        Ok(())
    }

    /// # Errors
    ///
    /// Will error if egui fails to render the partial view.
    ///
    /// # Panics
    ///
    /// Will panic if elements `Mutex` is poisoned.
    fn render_partial(
        &mut self,
        view: PartialView,
    ) -> Result<(), Box<dyn std::error::Error + Send + 'static>> {
        moosicbox_logging::debug_or_trace!(
            ("render_partial: start"),
            ("render_partial: start {:?}", view)
        );

        let mut page = self.app.container.write().unwrap();
        let ids = view
            .container
            .elements
            .as_slice()
            .iter()
            .filter_map(|x| x.container_element())
            .map(|x| x.id)
            .collect::<Vec<_>>();
        if let Some(removed) =
            page.replace_str_id_with_elements(view.container.elements, &view.target)
        {
            if let Some(container) = removed.container_element() {
                let mut visibilities = self.app.visibilities.write().unwrap();
                if let Some(visibility) = visibilities.remove(&container.id) {
                    for id in ids {
                        visibilities.insert(id, visibility);
                    }
                }
                drop(visibilities);
            }
            page.calc();
            drop(page);
            if let Some(ctx) = &*self.app.ctx.read().unwrap() {
                ctx.request_repaint();
            }
        } else {
            log::warn!("Unable to find element with id {}", view.target);
        }

        Ok(())
    }

    /// # Errors
    ///
    /// Will error if egui fails to render the canvas update.
    ///
    /// # Panics
    ///
    /// Will panic if elements `Mutex` is poisoned.
    fn render_canvas(
        &mut self,
        mut update: CanvasUpdate,
    ) -> Result<(), Box<dyn std::error::Error + Send + 'static>> {
        log::trace!("render_canvas: start");

        let mut binding = self.app.canvas_actions.write().unwrap();

        let actions = binding
            .entry(update.target)
            .or_insert_with(|| Vec::with_capacity(update.canvas_actions.len()));

        actions.append(&mut update.canvas_actions);

        compact_canvas_actions(actions);

        drop(binding);

        if let Some(ctx) = &*self.app.ctx.read().unwrap() {
            ctx.request_repaint();
        }

        log::trace!("render_canvas: end");
        Ok(())
    }

    fn container(&self) -> RwLockReadGuard<ContainerElement> {
        self.app.container.read().unwrap()
    }

    fn container_mut(&self) -> RwLockWriteGuard<ContainerElement> {
        self.app.container.write().unwrap()
    }
}

fn compact_canvas_actions(actions: &mut Vec<CanvasAction>) {
    let len = actions.len();
    for i in 0..len {
        let i = len - 1 - i;
        if matches!(actions[i], CanvasAction::Clear) {
            actions.drain(..=i);
            return;
        }
    }
}

#[derive(Debug)]
enum AppEvent {
    LoadImage { source: String },
    ProcessRoute { route: Route, container_id: usize },
}

#[derive(Clone)]
enum AppImage {
    Loading,
    Bytes(Arc<[u8]>),
}

#[derive(Clone)]
struct EguiApp {
    ctx: Arc<RwLock<Option<egui::Context>>>,
    width: Arc<RwLock<Option<f32>>>,
    height: Arc<RwLock<Option<f32>>>,
    container: Arc<RwLock<ContainerElement>>,
    sender: Sender<String>,
    event: Sender<AppEvent>,
    event_receiver: Receiver<AppEvent>,
    viewport_listeners: Arc<RwLock<HashMap<usize, ViewportListener>>>,
    images: Arc<RwLock<HashMap<String, AppImage>>>,
    canvas_actions: Arc<RwLock<HashMap<String, Vec<CanvasAction>>>>,
    route_requests: Arc<RwLock<Vec<usize>>>,
    visibilities: Arc<RwLock<HashMap<usize, Visibility>>>,
    checkboxes: Arc<RwLock<HashMap<egui::Id, bool>>>,
    router: Router,
    background: Option<Color32>,
    request_action: Sender<String>,
    on_resize: Sender<(f32, f32)>,
    side_effects: Arc<Mutex<VecDeque<Handler>>>,
}

type Handler = Box<dyn Fn() -> bool + Send + Sync>;

impl EguiApp {
    fn new(
        router: Router,
        sender: Sender<String>,
        event: Sender<AppEvent>,
        event_receiver: Receiver<AppEvent>,
        request_action: Sender<String>,
        on_resize: Sender<(f32, f32)>,
    ) -> Self {
        Self {
            ctx: Arc::new(RwLock::new(None)),
            width: Arc::new(RwLock::new(None)),
            height: Arc::new(RwLock::new(None)),
            container: Arc::new(RwLock::new(ContainerElement::default())),
            sender,
            event,
            event_receiver,
            viewport_listeners: Arc::new(RwLock::new(HashMap::new())),
            images: Arc::new(RwLock::new(HashMap::new())),
            canvas_actions: Arc::new(RwLock::new(HashMap::new())),
            route_requests: Arc::new(RwLock::new(vec![])),
            visibilities: Arc::new(RwLock::new(HashMap::new())),
            checkboxes: Arc::new(RwLock::new(HashMap::new())),
            router,
            background: None,
            request_action,
            on_resize,
            side_effects: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    #[allow(clippy::too_many_lines)]
    async fn listen(&self) {
        while let Ok(event) = self.event_receiver.recv_async().await {
            log::trace!("received event {event:?}");
            match event {
                AppEvent::LoadImage { source } => {
                    let images = self.images.clone();
                    let ctx = self.ctx.clone();
                    if let Some(bytes) = moosicbox_app_native_image::get_image(&source) {
                        log::trace!("loading image {source}");
                        images
                            .write()
                            .unwrap()
                            .insert(source, AppImage::Bytes(bytes.to_vec().into()));

                        if let Some(ctx) = &*ctx.read().unwrap() {
                            ctx.request_repaint();
                        }
                    } else {
                        moosicbox_task::spawn("renderer: load_image", async move {
                            static CLIENT: LazyLock<reqwest::Client> =
                                LazyLock::new(reqwest::Client::new);

                            log::trace!("loading image {source}");
                            match CLIENT.get(&source).send().await {
                                Ok(response) => {
                                    if !response.status().is_success() {
                                        log::error!(
                                            "Failed to load image: {}",
                                            response.text().await.unwrap_or_else(|e| {
                                                format!("(failed to get response text: {e:?})")
                                            })
                                        );
                                        return;
                                    }

                                    match response.bytes().await {
                                        Ok(bytes) => {
                                            let bytes = bytes.to_vec().into();

                                            let mut binding = images.write().unwrap();
                                            binding.insert(source, AppImage::Bytes(bytes));
                                            drop(binding);

                                            if let Some(ctx) = &*ctx.read().unwrap() {
                                                ctx.request_repaint();
                                            }
                                        }
                                        Err(e) => {
                                            log::error!("Failed to fetch image ({source}): {e:?}");
                                        }
                                    }
                                }
                                Err(e) => {
                                    log::error!("Failed to fetch image ({source}): {e:?}");
                                }
                            }
                        });
                    }
                }
                AppEvent::ProcessRoute {
                    route,
                    container_id,
                } => {
                    let router = self.router.clone();
                    let container = self.container.clone();
                    let ctx = self.ctx.clone();
                    moosicbox_task::spawn("renderer: ProcessRoute", async move {
                        match route {
                            Route::Get { route, trigger } | Route::Post { route, trigger } => {
                                if trigger.as_deref() == Some("load") {
                                    match router.navigate(&route).await {
                                        Ok(result) => {
                                            let ids = {
                                                let ids = result
                                                    .immediate
                                                    .elements
                                                    .iter()
                                                    .filter_map(|x| x.container_element())
                                                    .map(|x| x.id)
                                                    .collect_vec();
                                                log::debug!("ProcessRoute: replacing container_id={container_id} with {} elements", result.immediate.elements.len());
                                                let mut page = container.write().unwrap();
                                                if page.replace_id_with_elements(
                                                    result.immediate.elements,
                                                    container_id,
                                                ) {
                                                    page.calc();
                                                    drop(page);
                                                    if let Some(ctx) = &*ctx.read().unwrap() {
                                                        ctx.request_repaint();
                                                    }
                                                } else {
                                                    log::warn!("Unable to find element with id {container_id}");
                                                }
                                                ids
                                            };
                                            {
                                                if let Some(future) = result.future {
                                                    let elements = future.await;
                                                    let mut page = container.write().unwrap();
                                                    if page.replace_ids_with_elements(
                                                        elements.elements,
                                                        &ids,
                                                    ) {
                                                        page.calc();
                                                        drop(page);
                                                        if let Some(ctx) = &*ctx.read().unwrap() {
                                                            ctx.request_repaint();
                                                        }
                                                    } else {
                                                        log::warn!("Unable to find element with ids {ids:?}");
                                                    }
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            log::error!("Failed to process route ({route}): {e:?}");
                                        }
                                    }
                                }
                            }
                        }
                    });
                }
            }
        }
    }

    fn update_frame_size(&self, width: f32, height: f32) {
        *self.viewport_listeners.write().unwrap() = HashMap::new();

        log::debug!(
            "calc: frame size changed from ({:?}, {:?}) -> ({width}, {height})",
            self.width.read().unwrap(),
            self.height.read().unwrap()
        );

        {
            let mut container = self.container.write().unwrap();
            container.calculated_width.replace(width);
            container.calculated_height.replace(height);
            container.calc();
        }

        self.width.write().unwrap().replace(width);
        self.height.write().unwrap().replace(height);
    }

    fn calc(&self, ctx: &egui::Context) {
        ctx.input(move |i| {
            let width = i.screen_rect.width();
            let height = i.screen_rect.height();
            let current_width = *self.width.read().unwrap();
            let current_height = *self.height.read().unwrap();
            if current_width.is_none_or(|x| (x - width).abs() >= 0.01)
                || current_height.is_none_or(|x| (x - height).abs() >= 0.01)
            {
                self.update_frame_size(width, height);
                if let Err(e) = self.on_resize.send((width, height)) {
                    moosicbox_assert::die_or_error!(
                        "Failed to send on_resize message: {width}, {height}: {e:?}"
                    );
                }
            }
        });
    }

    fn get_scroll_container(
        rect: egui::Rect,
        pos_x: f32,
        pos_y: f32,
        element: &ContainerElement,
        parent: Option<&Viewport>,
    ) -> Viewport {
        let viewport = Viewport {
            parent: parent.cloned().map(Box::new),
            pos: Pos {
                x: pos_x,
                y: pos_y,
                w: element.calculated_width.unwrap(),
                h: element.calculated_height.unwrap(),
            },
            viewport: Pos {
                x: rect.min.x,
                y: rect.min.y,
                w: element.calculated_width.unwrap(),
                h: element.calculated_height.unwrap(),
            },
        };

        log::trace!(
            "get_scroll_container: ({}, {})",
            viewport.pos.x,
            viewport.pos.y
        );

        viewport
    }

    fn render_horizontal_borders(
        ui: &mut Ui,
        container: &ContainerElement,
        add_contents: impl FnOnce(&mut Ui) -> Response,
    ) -> Response {
        ui.horizontal(|ui| {
            if let Some((color, size)) = container.calculated_border_left {
                egui::Frame::none().fill(color.into()).show(ui, |ui| {
                    ui.set_width(size);
                    ui.set_height(container.calculated_height.unwrap_or(0.0));
                });
            }

            let response = add_contents(ui);

            if let Some((color, size)) = container.calculated_border_right {
                egui::Frame::none().fill(color.into()).show(ui, |ui| {
                    ui.set_width(size);
                    ui.set_height(container.calculated_height.unwrap_or(0.0));
                });
            }

            response
        })
        .inner
    }

    fn render_vertical_borders(
        ui: &mut Ui,
        container: &ContainerElement,
        add_contents: impl FnOnce(&mut Ui) -> Response,
    ) -> Response {
        ui.vertical(|ui| {
            if let Some((color, size)) = container.calculated_border_top {
                egui::Frame::none().fill(color.into()).show(ui, |ui| {
                    ui.set_width(container.calculated_width.unwrap_or(0.0));
                    ui.set_height(size);
                });
            }

            let response = add_contents(ui);

            if let Some((color, size)) = container.calculated_border_bottom {
                egui::Frame::none().fill(color.into()).show(ui, |ui| {
                    ui.set_width(container.calculated_width.unwrap_or(0.0));
                    ui.set_height(size);
                });
            }

            response
        })
        .inner
    }

    fn render_borders(
        ui: &mut Ui,
        container: &ContainerElement,
        add_contents: impl FnOnce(&mut Ui) -> Response,
    ) -> Response {
        if container.calculated_border_left.is_some() || container.calculated_border_right.is_some()
        {
            Self::render_horizontal_borders(ui, container, |ui| {
                if container.calculated_border_top.is_some()
                    || container.calculated_border_bottom.is_some()
                {
                    Self::render_vertical_borders(ui, container, add_contents)
                } else {
                    add_contents(ui)
                }
            })
        } else if container.calculated_border_top.is_some()
            || container.calculated_border_bottom.is_some()
        {
            Self::render_vertical_borders(ui, container, add_contents)
        } else {
            add_contents(ui)
        }
    }

    fn container_hidden(&self, container: &ContainerElement) -> bool {
        if container.visibility == Some(Visibility::Hidden) {
            self.visibilities
                .read()
                .unwrap()
                .get(&container.id)
                .copied()
                .unwrap_or(Visibility::Hidden)
                == Visibility::Hidden
        } else {
            false
        }
    }

    fn handle_scroll_child_out_of_view(
        &self,
        ctx: &egui::Context,
        ui: &mut Ui,
        container: &ContainerElement,
        viewport: Option<&Viewport>,
        rect: Option<egui::Rect>,
        relative_container: Option<(egui::Rect, &ContainerElement)>,
    ) -> bool {
        if let Some(rect) = rect {
            let render_rect = Self::get_render_rect(ui, container, relative_container);
            let width = render_rect.width();
            let height = render_rect.height();
            let (offset_x, offset_y) =
                viewport.map_or((0.0, 0.0), |viewport| (viewport.pos.x, viewport.pos.y));

            if render_rect.min.x + width - offset_x < -1.0
                || render_rect.min.y + height - offset_y < -1.0
                || render_rect.min.x - offset_x >= rect.width() + 1.0
                || render_rect.min.y - offset_y >= rect.height() + 1.0
            {
                log::trace!(
                    "render_container: skipping ({}, {}, {width}, {height})",
                    render_rect.min.x,
                    render_rect.min.y
                );
                self.handle_container_side_effects(
                    ctx,
                    Some(ui),
                    container,
                    viewport,
                    Some(rect),
                    None,
                    true,
                );
                ui.allocate_space(egui::vec2(width, height));
                return true;
            }
            log::trace!(
                "render_container: showing ({}, {}, {width}, {height})",
                render_rect.min.x,
                render_rect.min.y
            );
        }

        false
    }

    #[allow(clippy::too_many_lines, clippy::too_many_arguments)]
    fn render_container(
        &self,
        ctx: &egui::Context,
        ui: &mut Ui,
        container: &ContainerElement,
        viewport: Option<&Viewport>,
        rect: Option<egui::Rect>,
        relative_container: Option<(egui::Rect, &ContainerElement)>,
    ) -> Option<Response> {
        if container.is_hidden() {
            log::debug!("render_container: container is hidden. skipping render");
            self.handle_container_side_effects(
                ctx,
                Some(ui),
                container,
                viewport,
                rect,
                None,
                true,
            );
            return None;
        }

        if self.container_hidden(container) {
            let response = ui
                .allocate_new_ui(
                    egui::UiBuilder::new().max_rect(Self::get_render_rect(
                        ui,
                        container,
                        relative_container,
                    )),
                    |ui| {
                        ui.set_width(container.calculated_width.unwrap());
                        ui.set_height(container.calculated_height.unwrap());
                    },
                )
                .response;

            self.handle_container_side_effects(
                ctx,
                None,
                container,
                viewport,
                rect,
                Some(&response),
                true,
            );

            return Some(response);
        }

        Some(Self::render_borders(ui, container, |ui| {
            egui::Frame::none().inner_margin(egui::Margin {
                    left: container.internal_margin_left.unwrap_or(0.0),
                    right: container.internal_margin_right.unwrap_or(0.0),
                    top: container.internal_margin_top.unwrap_or(0.0),
                    bottom: container.internal_margin_bottom.unwrap_or(0.0),
                })
                .show(ui, {
                    move |ui| {
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                            egui::Frame::none().show(ui, {
                                move |ui| {
                                    let cursor = ui.cursor();
                                    let (pos_x, pos_y) = (cursor.left(), cursor.top());
                                    match (container.overflow_x, container.overflow_y) {
                                        (
                                            LayoutOverflow::Auto,
                                            LayoutOverflow::Auto,
                                        ) => {
                                            egui::ScrollArea::both()
                                                .scroll_bar_visibility(
                                                    egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded,
                                                )
                                                .show_viewport(ui, {
                                                    move |ui, rect| {
                                                        let viewport = Self::get_scroll_container(rect, pos_x, pos_y, container, viewport);
                                                        let viewport = Some(&viewport);
                                                        self.render_container_contents(
                                                            ctx,
                                                            ui,
                                                            container,
                                                            viewport,
                                                            Some(rect),
                                                            relative_container,
                                                            true,
                                                        )
                                                    }
                                                }).inner
                                        }
                                        (
                                            LayoutOverflow::Scroll,
                                            LayoutOverflow::Scroll,
                                        ) => {
                                            egui::ScrollArea::both()
                                                .scroll_bar_visibility(
                                                    egui::scroll_area::ScrollBarVisibility::AlwaysVisible,
                                                )
                                                .show_viewport(ui, {
                                                    move |ui, rect| {
                                                        let viewport = Self::get_scroll_container(rect, pos_x, pos_y, container, viewport);
                                                        let viewport = Some(&viewport);
                                                        self.render_container_contents(
                                                            ctx,
                                                            ui,
                                                            container,
                                                            viewport,
                                                            Some(rect),
                                                            relative_container,
                                                            true,
                                                        )
                                                    }
                                                }).inner
                                        }
                                        (
                                            LayoutOverflow::Auto,
                                            LayoutOverflow::Scroll,
                                        ) => {
                                            egui::ScrollArea::vertical()
                                                .scroll_bar_visibility(
                                                    egui::scroll_area::ScrollBarVisibility::AlwaysVisible,
                                                )
                                                .show_viewport(ui, {
                                                    move |ui, rect| {
                                                        let viewport = Self::get_scroll_container(rect, pos_x, pos_y, container, viewport);
                                                        let viewport = Some(&viewport);
                                                        let cursor = ui.cursor();
                                                        let (pos_x, pos_y) = (cursor.left(), cursor.top());
                                                        egui::ScrollArea::horizontal()
                                                            .scroll_bar_visibility(
                                                                egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded,
                                                            )
                                                            .show_viewport(ui, {
                                                                move |ui, rect| {
                                                                    let viewport = Self::get_scroll_container(rect, pos_x, pos_y, container, viewport);
                                                                    let viewport = Some(&viewport);
                                                                    self.render_container_contents(
                                                                        ctx,
                                                                        ui,
                                                                        container,
                                                                        viewport,
                                                                        Some(rect),
                                                                        relative_container,
                                                                        true,
                                                                    )
                                                                }
                                                            }).inner
                                                    }
                                                }).inner
                                        }
                                        (
                                            LayoutOverflow::Scroll,
                                            LayoutOverflow::Auto,
                                        ) => {
                                            egui::ScrollArea::vertical()
                                                .scroll_bar_visibility(
                                                    egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded,
                                                )
                                                .show_viewport(ui, {
                                                    move |ui, rect| {
                                                        let viewport = Self::get_scroll_container(rect, pos_x, pos_y, container, viewport);
                                                        let viewport = Some(&viewport);
                                                        let cursor = ui.cursor();
                                                        let (pos_x, pos_y) = (cursor.left(), cursor.top());
                                                        egui::ScrollArea::horizontal()
                                                            .scroll_bar_visibility(
                                                                egui::scroll_area::ScrollBarVisibility::AlwaysVisible,
                                                            )
                                                            .show_viewport(ui, {
                                                                move |ui, rect| {
                                                                let viewport = Self::get_scroll_container(rect, pos_x, pos_y, container, viewport);
                                                                let viewport = Some(&viewport);
                                                                    self.render_container_contents(
                                                                        ctx,
                                                                        ui,
                                                                        container,
                                                                        viewport,
                                                                        Some(rect),
                                                                        relative_container,
                                                                        true,
                                                                    )
                                                                }
                                                            }).inner
                                                    }
                                                }).inner
                                        }
                                        (LayoutOverflow::Auto, _) => {
                                            egui::ScrollArea::horizontal()
                                                .scroll_bar_visibility(
                                                    egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded,
                                                )
                                                .show_viewport(ui, {
                                                    move |ui, rect| {
                                                        let viewport = Self::get_scroll_container(rect, pos_x, pos_y, container, viewport);
                                                        let viewport = Some(&viewport);
                                                        self.render_container_contents(
                                                            ctx,
                                                            ui,
                                                            container,
                                                            viewport,
                                                            Some(rect),
                                                            relative_container,
                                                            false,
                                                        )
                                                    }
                                                }).inner
                                        }
                                        (LayoutOverflow::Scroll, _) => {
                                            egui::ScrollArea::horizontal()
                                                .scroll_bar_visibility(
                                                    egui::scroll_area::ScrollBarVisibility::AlwaysVisible,
                                                )
                                                .show_viewport(ui, {
                                                    move |ui, rect| {
                                                        let viewport = Self::get_scroll_container(rect, pos_x, pos_y, container, viewport);
                                                        let viewport = Some(&viewport);
                                                        self.render_container_contents(
                                                            ctx,
                                                            ui,
                                                            container,
                                                            viewport,
                                                            Some(rect),
                                                            relative_container,
                                                            false,
                                                        )
                                                    }
                                                }).inner
                                        }
                                        (_, LayoutOverflow::Auto) => {
                                            egui::ScrollArea::vertical()
                                                .scroll_bar_visibility(
                                                    egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded,
                                                )
                                                .show_viewport(ui, {
                                                    move |ui, rect| {
                                                        let viewport = Self::get_scroll_container(rect, pos_x, pos_y, container, viewport);
                                                        let viewport = Some(&viewport);
                                                        self.render_container_contents(
                                                            ctx,
                                                            ui,
                                                            container,
                                                            viewport,
                                                            Some(rect),
                                                            relative_container,
                                                            true,
                                                        )
                                                    }
                                                }).inner
                                        }
                                        (_, LayoutOverflow::Scroll) => {
                                            egui::ScrollArea::vertical()
                                                .scroll_bar_visibility(
                                                    egui::scroll_area::ScrollBarVisibility::AlwaysVisible,
                                                )
                                                .show_viewport(ui, {
                                                    move |ui, rect| {
                                                        let viewport = Self::get_scroll_container(rect, pos_x, pos_y, container, viewport);
                                                        let viewport = Some(&viewport);
                                                        self.render_container_contents(
                                                            ctx,
                                                            ui,
                                                            container,
                                                            viewport,
                                                            Some(rect),
                                                            relative_container,
                                                            true,
                                                        )
                                                    }
                                                }).inner
                                        }
                                        (_, _) => {
                                            self.render_container_contents(
                                                ctx,
                                                ui,
                                                container,
                                                viewport,
                                                rect,
                                                relative_container,
                                                false,
                                            )
                                        }
                                    }
                                }
                            })
                        })
                    }
                }).inner.inner.inner
        }))
    }

    fn get_render_rect(
        ui: &Ui,
        container: &ContainerElement,
        relative_container: Option<(egui::Rect, &ContainerElement)>,
    ) -> egui::Rect {
        if container.position == Some(Position::Absolute) {
            if let Some((relative_rect, ..)) = relative_container {
                return relative_rect
                    .with_min_x(relative_rect.min.x + container.calculated_x.unwrap())
                    .with_min_y(relative_rect.min.y + container.calculated_y.unwrap())
                    .with_max_x(
                        relative_rect.min.x
                            + container.calculated_x.unwrap()
                            + container.calculated_width.unwrap(),
                    )
                    .with_max_y(
                        relative_rect.min.y
                            + container.calculated_y.unwrap()
                            + container.calculated_height.unwrap(),
                    );
            }
        }

        egui::Rect::from_min_size(
            ui.cursor().left_top(),
            egui::vec2(
                container.calculated_width.unwrap(),
                container.calculated_height.unwrap(),
            ),
        )
    }

    fn render_position<'a>(
        ui: &mut Ui,
        container: &'a ContainerElement,
        mut relative_container: Option<(egui::Rect, &'a ContainerElement)>,
        inner: impl FnOnce(&mut Ui, Option<(egui::Rect, &'a ContainerElement)>) -> Response,
    ) -> Response {
        match container.position {
            Some(Position::Relative) => {
                let pos = ui.cursor().left_top();
                let size = egui::vec2(
                    container.calculated_width.unwrap(),
                    container.calculated_height.unwrap(),
                );
                relative_container = Some((egui::Rect::from_min_size(pos, size), container));
            }
            Some(Position::Absolute) => {
                let abs_rect = Self::get_render_rect(ui, container, relative_container);

                return ui.put(abs_rect, |ui: &mut egui::Ui| inner(ui, relative_container));
            }
            Some(Position::Static) | None => {}
        }

        inner(ui, relative_container)
    }

    #[allow(clippy::too_many_lines, clippy::too_many_arguments)]
    fn render_direction<'a>(
        &self,
        ctx: &egui::Context,
        ui: &mut Ui,
        container: &'a ContainerElement,
        viewport: Option<&Viewport>,
        rect: Option<egui::Rect>,
        relative_container: Option<(egui::Rect, &'a ContainerElement)>,
        vscroll: bool,
    ) -> Response {
        match container.direction {
            LayoutDirection::Row => {
                let rows = container
                    .elements
                    .iter()
                    .filter_map(|x| x.container_element().map(|y| (x, y)))
                    .filter_map(|(x, y)| y.calculated_position.as_ref().map(|y| (x, y)))
                    .filter_map(|(x, y)| match y {
                        LayoutPosition::Wrap { row, .. } => Some((*row, x)),
                        LayoutPosition::Default => {
                            self.handle_element_side_effects(
                                ctx, None, x, viewport, rect, None, true,
                            );
                            None
                        }
                    })
                    .chunk_by(|(row, _element)| *row);

                let mut rows = rows
                    .into_iter()
                    .map(|(_row, y)| y.into_iter().map(|(_, element)| element).collect_vec())
                    .peekable();

                if rows.peek().is_some() {
                    ui.vertical(move |ui| {
                        for row in rows {
                            ui.horizontal(move |ui| {
                                self.render_elements_ref(
                                    ctx,
                                    ui,
                                    &row,
                                    viewport,
                                    rect,
                                    relative_container,
                                    !vscroll && rect.is_some(),
                                );
                            });
                        }
                    })
                    .response
                } else {
                    ui.horizontal(move |ui| {
                        self.render_elements(
                            ctx,
                            ui,
                            &container.elements,
                            viewport,
                            rect,
                            relative_container,
                            !vscroll && rect.is_some(),
                        );
                    })
                    .response
                }
            }
            LayoutDirection::Column => {
                let cols = container
                    .elements
                    .iter()
                    .filter_map(|x| x.container_element().map(|y| (x, y)))
                    .filter_map(|(x, y)| y.calculated_position.as_ref().map(|y| (x, y)))
                    .filter_map(|(x, y)| match y {
                        LayoutPosition::Wrap { col, .. } => Some((*col, x)),
                        LayoutPosition::Default => {
                            self.handle_element_side_effects(
                                ctx, None, x, viewport, rect, None, true,
                            );
                            None
                        }
                    })
                    .chunk_by(|(col, _element)| *col);

                let mut cols = cols
                    .into_iter()
                    .map(|(_row, y)| y.into_iter().map(|(_, element)| element).collect_vec())
                    .peekable();

                if cols.peek().is_some() {
                    ui.horizontal(move |ui| {
                        for col in cols {
                            ui.vertical(move |ui| {
                                self.render_elements_ref(
                                    ctx,
                                    ui,
                                    &col,
                                    viewport,
                                    rect,
                                    relative_container,
                                    !vscroll && rect.is_some(),
                                );
                            });
                        }
                    })
                    .response
                } else {
                    ui.vertical(move |ui| {
                        self.render_elements(
                            ctx,
                            ui,
                            &container.elements,
                            viewport,
                            rect,
                            relative_container,
                            !vscroll && rect.is_some(),
                        );
                    })
                    .response
                }
            }
        }
    }

    fn render_layout<'a>(
        ui: &mut Ui,
        container: &'a ContainerElement,
        relative_container: Option<(egui::Rect, &'a ContainerElement)>,
        inner: impl FnOnce(&mut Ui, Option<(egui::Rect, &'a ContainerElement)>) -> Response,
    ) -> Response {
        match container.justify_content {
            JustifyContent::Center => {
                ui.allocate_new_ui(
                    egui::UiBuilder::new().layout(match container.align_items {
                        AlignItems::Center => {
                            egui::Layout::centered_and_justified(egui::Direction::TopDown)
                        }
                        AlignItems::End | AlignItems::Default => {
                            egui::Layout::top_down_justified(egui::Align::Center)
                        }
                    }),
                    |ui| {
                        egui::Frame::none().show(ui, |ui| {
                            ui.set_width(container.contained_calculated_width());
                            ui.set_height(container.contained_calculated_height());
                            if container.align_items == AlignItems::End {
                                let rect = egui::Rect::from_min_size(
                                    ui.cursor().left_top(),
                                    egui::vec2(
                                        0.0,
                                        container.calculated_height.unwrap()
                                            - container.contained_calculated_height(),
                                    ),
                                );
                                ui.advance_cursor_after_rect(rect);
                            }

                            inner(ui, relative_container)
                        })
                    },
                )
                .response
            }
            JustifyContent::End => {
                ui.add_space(
                    container.calculated_width.unwrap() - container.contained_calculated_width(),
                );

                inner(ui, relative_container)
            }
            _ => inner(ui, relative_container),
        }
    }

    #[allow(clippy::too_many_arguments, clippy::too_many_lines)]
    fn render_container_contents<'a>(
        &self,
        ctx: &egui::Context,
        ui: &mut Ui,
        container: &'a ContainerElement,
        viewport: Option<&Viewport>,
        rect: Option<egui::Rect>,
        relative_container: Option<(egui::Rect, &'a ContainerElement)>,
        vscroll: bool,
    ) -> Response {
        Self::render_position(
            ui,
            container,
            relative_container,
            |ui, relative_container| {
                let mut frame = egui::Frame::none();

                if let Some(background) = container.background {
                    frame = frame.fill(background.into());
                }
                if container.calculated_border_top_left_radius.is_some()
                    || container.calculated_border_top_right_radius.is_some()
                    || container.calculated_border_bottom_left_radius.is_some()
                    || container.calculated_border_bottom_right_radius.is_some()
                {
                    frame = frame.rounding(egui::Rounding {
                        nw: container.calculated_border_top_left_radius.unwrap_or(0.0),
                        ne: container.calculated_border_top_right_radius.unwrap_or(0.0),
                        sw: container
                            .calculated_border_bottom_left_radius
                            .unwrap_or(0.0),
                        se: container
                            .calculated_border_bottom_right_radius
                            .unwrap_or(0.0),
                    });
                }

                frame
                    .show(ui, {
                        |ui| {
                            if let Some(width) = container.calculated_width {
                                ui.set_width(width);
                            }
                            if let Some(height) = container.calculated_height {
                                ui.set_height(height);

                                if vscroll {
                                    if ctx.input(|i| i.key_pressed(egui::Key::PageDown)) {
                                        let rect =
                                            egui::Rect::from_pos(egui::emath::pos2(0.0, height));
                                        ui.scroll_to_rect(rect, Some(egui::Align::TOP));
                                    }
                                    if ctx.input(|i| i.key_pressed(egui::Key::PageUp)) {
                                        let rect =
                                            egui::Rect::from_pos(egui::emath::pos2(0.0, -height));
                                        ui.scroll_to_rect(rect, Some(egui::Align::TOP));
                                    }
                                }
                            }

                            Self::render_layout(
                                ui,
                                container,
                                relative_container,
                                move |ui, relative_container| {
                                    self.render_direction(
                                        ctx,
                                        ui,
                                        container,
                                        viewport,
                                        rect,
                                        relative_container,
                                        vscroll,
                                    )
                                },
                            )
                        }
                    })
                    .response
            },
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn render_elements(
        &self,
        ctx: &egui::Context,
        ui: &mut Ui,
        elements: &[Element],
        viewport: Option<&Viewport>,
        rect: Option<egui::Rect>,
        relative_container: Option<(egui::Rect, &ContainerElement)>,
        scroll_child: bool,
    ) {
        log::trace!("render_elements: {} elements", elements.len());
        for element in elements {
            self.render_element(
                ctx,
                ui,
                element,
                viewport,
                rect,
                relative_container,
                scroll_child,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_elements_ref(
        &self,
        ctx: &egui::Context,
        ui: &mut Ui,
        elements: &[&Element],
        viewport: Option<&Viewport>,
        rect: Option<egui::Rect>,
        relative_container: Option<(egui::Rect, &ContainerElement)>,
        scroll_child: bool,
    ) {
        log::trace!("render_elements_ref: {} elements", elements.len());
        for element in elements {
            self.render_element(
                ctx,
                ui,
                element,
                viewport,
                rect,
                relative_container,
                scroll_child,
            );
        }
    }

    fn rect_contains_mouse(
        pointer: &egui::PointerState,
        rect: egui::Rect,
        viewport: Option<egui::Rect>,
    ) -> bool {
        pointer.latest_pos().is_some_and(|pos| {
            if viewport.is_some_and(|vp| !vp.contains(pos)) {
                return false;
            }
            rect.contains(pos)
        })
    }

    #[allow(clippy::too_many_lines, clippy::too_many_arguments)]
    fn handle_container_side_effects(
        &self,
        ctx: &egui::Context,
        ui: Option<&Ui>,
        container: &ContainerElement,
        viewport: Option<&Viewport>,
        rect: Option<egui::Rect>,
        response: Option<&Response>,
        recurse: bool,
    ) {
        if let Some(route) = &container.route {
            let processed_route = {
                self.route_requests
                    .read()
                    .unwrap()
                    .iter()
                    .any(|x| *x == container.id)
            };
            if !processed_route {
                log::debug!(
                    "processing route route={route:?} container_id={}",
                    container.id
                );
                self.route_requests.write().unwrap().push(container.id);
                if let Err(e) = self.event.send(AppEvent::ProcessRoute {
                    route: route.to_owned(),
                    container_id: container.id,
                }) {
                    log::error!("Failed to send ProcessRoute event: {e:?}");
                }
            }
        }
        if let Some(response) = response {
            let viewport_rect = rect.map(|rect| {
                let (offset_x, offset_y) =
                    viewport.map_or((0.0, 0.0), |viewport| (viewport.pos.x, viewport.pos.y));
                egui::Rect::from_min_size(egui::pos2(offset_x, offset_y), rect.size())
            });

            if let Some(cursor) = container.cursor {
                let ctx = ctx.clone();
                let pointer = ctx.input(|x| x.pointer.clone());
                let response = response.clone();
                self.trigger_side_effect(move || {
                    if Self::rect_contains_mouse(&pointer, response.rect, viewport_rect) {
                        ctx.output_mut(|x| {
                            x.cursor_icon = cursor_to_cursor_icon(cursor);
                        });
                    }

                    true
                });
            }
            if container.is_visible() {
                for fx_action in &container.actions {
                    let request_action = self.request_action.clone();

                    match fx_action.trigger {
                        ActionTrigger::Click | ActionTrigger::ClickOutside => {
                            let inside = matches!(fx_action.trigger, ActionTrigger::Click);
                            let action = fx_action.action.clone();
                            let id = container.id;
                            let visibilities = self.visibilities.clone();
                            let pointer = ctx.input(|x| x.pointer.clone());
                            let response = response.clone();
                            let container = self.container.clone();
                            let sender = self.sender.clone();
                            self.trigger_side_effect(move || {
                                if Self::rect_contains_mouse(&pointer, response.rect, viewport_rect)
                                    == inside
                                    && pointer.primary_released()
                                {
                                    log::trace!("click action: {action}");
                                    Self::handle_action(
                                        &action,
                                        id,
                                        &sender,
                                        &container,
                                        &visibilities,
                                        &request_action,
                                    );
                                    return !inside;
                                }

                                Self::unhandle_action(&action, id, &visibilities);

                                true
                            });
                        }
                        ActionTrigger::Hover => {
                            let action = fx_action.action.clone();
                            let id = container.id;
                            let visibilities = self.visibilities.clone();
                            let response = response.clone();
                            let pointer = ctx.input(|x| x.pointer.clone());
                            let container = self.container.clone();
                            let sender = self.sender.clone();
                            self.trigger_side_effect(move || {
                                if Self::rect_contains_mouse(&pointer, response.rect, viewport_rect)
                                {
                                    log::trace!("hover action: {action}");
                                    return Self::handle_action(
                                        &action,
                                        id,
                                        &sender,
                                        &container,
                                        &visibilities,
                                        &request_action,
                                    );
                                }

                                Self::unhandle_action(&action, id, &visibilities);

                                true
                            });
                        }
                        ActionTrigger::Change => {
                            let action = fx_action.action.clone();
                            let id = container.id;
                            let visibilities = self.visibilities.clone();
                            let response = response.clone();
                            let container = self.container.clone();
                            let sender = self.sender.clone();
                            self.trigger_side_effect(move || {
                                if response.changed() {
                                    log::trace!("change action: {action}");
                                    return Self::handle_action(
                                        &action,
                                        id,
                                        &sender,
                                        &container,
                                        &visibilities,
                                        &request_action,
                                    );
                                }

                                Self::unhandle_action(&action, id, &visibilities);

                                true
                            });
                        }
                        ActionTrigger::Immediate => {}
                    }
                }
            }
        }

        if recurse {
            for element in &container.elements {
                self.handle_element_side_effects(
                    ctx, ui, element, viewport, rect, response, recurse,
                );
            }
        }
    }

    #[allow(clippy::too_many_lines, clippy::too_many_arguments)]
    fn handle_element_side_effects(
        &self,
        ctx: &egui::Context,
        ui: Option<&Ui>,
        element: &Element,
        viewport: Option<&Viewport>,
        rect: Option<egui::Rect>,
        response: Option<&Response>,
        recurse: bool,
    ) {
        log::trace!("handle_element_side_effects");
        if let Some(response) = response {
            let viewport_rect = rect.map(|rect| {
                let (offset_x, offset_y) =
                    viewport.map_or((0.0, 0.0), |viewport| (viewport.pos.x, viewport.pos.y));
                egui::Rect::from_min_size(egui::pos2(offset_x, offset_y), rect.size())
            });

            match element {
                Element::Button { .. } => {
                    let response = response.clone();
                    let pointer = ctx.input(|x| x.pointer.clone());
                    let ctx = ctx.clone();
                    self.trigger_side_effect(move || {
                        if Self::rect_contains_mouse(&pointer, response.rect, viewport_rect) {
                            ctx.output_mut(|x| x.cursor_icon = CursorIcon::PointingHand);
                        }

                        true
                    });
                }
                Element::Anchor { href, .. } => {
                    let href = href.to_owned();
                    let sender = self.sender.clone();
                    let response = response.clone();
                    let pointer = ctx.input(|x| x.pointer.clone());
                    let ctx = ctx.clone();
                    self.trigger_side_effect(move || {
                        if Self::rect_contains_mouse(&pointer, response.rect, viewport_rect)
                            && pointer.primary_released()
                        {
                            if let Some(href) = href.clone() {
                                if let Err(e) = sender.send(href) {
                                    log::error!("Failed to send href event: {e:?}");
                                }
                            }
                        }

                        if Self::rect_contains_mouse(&pointer, response.rect, viewport_rect) {
                            ctx.output_mut(|x| x.cursor_icon = CursorIcon::PointingHand);
                        }

                        true
                    });
                }
                _ => {}
            }
        }

        if let Some(ui) = ui {
            if let Element::Image {
                source: Some(source),
                element,
            } = element
            {
                let listeners: &mut HashMap<_, _> = &mut self.viewport_listeners.write().unwrap();

                let pos = ui.cursor().left_top();
                let listener = listeners.entry(element.id).or_insert_with(|| {
                    ViewportListener::new(
                        viewport.cloned(),
                        0.0,
                        0.0,
                        element.calculated_width.unwrap(),
                        element.calculated_height.unwrap(),
                    )
                });
                listener.viewport = viewport.cloned();
                listener.pos.x = pos.x + viewport.map_or(0.0, |x| x.viewport.x);
                listener.pos.y = pos.y + viewport.map_or(0.0, |x| x.viewport.y);

                let (_, (dist, prev_dist)) = listener.check();

                if prev_dist.is_none_or(|x| x >= 2000.0) && dist < 2000.0 {
                    let contains_image = {
                        matches!(
                            self.images.read().unwrap().get(source),
                            Some(AppImage::Bytes(_))
                        )
                    };
                    if !contains_image {
                        let loading_image = {
                            matches!(
                                self.images.read().unwrap().get(source),
                                Some(AppImage::Loading)
                            )
                        };

                        if !loading_image {
                            log::debug!(
                                "render_element: triggering LoadImage for source={source} ({}, {})",
                                listener.pos.x,
                                listener.pos.y
                            );
                            self.images
                                .write()
                                .unwrap()
                                .insert(source.clone(), AppImage::Loading);

                            if let Err(e) = self.event.send(AppEvent::LoadImage {
                                source: source.to_string(),
                            }) {
                                log::error!("Failed to send LoadImage event: {e:?}");
                            }
                        }
                    }
                }
            }
        }

        if let Some(container) = element.container_element() {
            self.handle_container_side_effects(
                ctx, ui, container, viewport, rect, response, recurse,
            );
        }
    }

    #[allow(clippy::too_many_lines)]
    fn handle_action(
        action: &ActionType,
        id: usize,
        sender: &Sender<String>,
        container: &RwLock<ContainerElement>,
        visibilities: &RwLock<HashMap<usize, Visibility>>,
        request_action: &Sender<String>,
    ) -> bool {
        log::trace!("handle_action: action={action}");
        match &action {
            ActionType::Style { target, action } => {
                Self::handle_style_action(action, target, id, container, visibilities)
            }
            ActionType::Navigate { url } => {
                if let Err(e) = sender.send(url.to_owned()) {
                    log::error!("Failed to navigate via action: {e:?}");
                }
                true
            }
            ActionType::Log { message, level } => {
                match level {
                    gigachad_actions::LogLevel::Error => {
                        log::error!("{message}");
                    }
                    gigachad_actions::LogLevel::Warn => {
                        log::warn!("{message}");
                    }
                    gigachad_actions::LogLevel::Info => {
                        log::info!("{message}");
                    }
                    gigachad_actions::LogLevel::Debug => {
                        log::debug!("{message}");
                    }
                    gigachad_actions::LogLevel::Trace => {
                        log::trace!("{message}");
                    }
                }

                true
            }
            ActionType::Custom { action } => {
                if let Err(e) = request_action.send(action.clone()) {
                    moosicbox_assert::die_or_error!("Failed to request action: {action} ({e:?})");
                }
                true
            }
            ActionType::Logic(eval) => {
                let calc_visibility =
                    |calc_value: &gigachad_actions::logic::CalcValue| match calc_value {
                        gigachad_actions::logic::CalcValue::GetVisibility { target } => {
                            match target {
                                gigachad_actions::ElementTarget::StrId(str_id) => container
                                    .read()
                                    .unwrap()
                                    .find_element_by_str_id(str_id)
                                    .and_then(|x| x.container_element())
                                    .and_then(|element| {
                                        visibilities
                                            .read()
                                            .unwrap()
                                            .get(&element.id)
                                            .copied()
                                            .or(element.visibility)
                                    })
                                    .unwrap_or_default(),
                                gigachad_actions::ElementTarget::Id(id) => visibilities
                                    .write()
                                    .unwrap()
                                    .get(id)
                                    .copied()
                                    .unwrap_or_default(),
                                gigachad_actions::ElementTarget::SelfTarget => visibilities
                                    .write()
                                    .unwrap()
                                    .get(&id)
                                    .copied()
                                    .unwrap_or_default(),
                            }
                        }
                    };

                let success = match &eval.condition {
                    gigachad_actions::logic::Condition::Eq(a, b) => match a {
                        gigachad_actions::logic::Value::Calc(calc_value) => match b {
                            gigachad_actions::logic::Value::Calc(b_calc_value) => {
                                calc_visibility(calc_value) == calc_visibility(b_calc_value)
                            }
                            gigachad_actions::logic::Value::Visibility(visibility) => {
                                *visibility == calc_visibility(calc_value)
                            }
                        },
                        gigachad_actions::logic::Value::Visibility(visibility) => match b {
                            gigachad_actions::logic::Value::Calc(calc_value) => {
                                *visibility == calc_visibility(calc_value)
                            }
                            gigachad_actions::logic::Value::Visibility(b_visibility) => {
                                visibility == b_visibility
                            }
                        },
                    },
                };

                if success {
                    for action in &eval.actions {
                        if action.trigger == ActionTrigger::Immediate
                            && !Self::handle_action(
                                &action.action,
                                id,
                                sender,
                                container,
                                visibilities,
                                request_action,
                            )
                        {
                            return false;
                        }
                    }
                } else {
                    for action in &eval.else_actions {
                        if action.trigger == ActionTrigger::Immediate
                            && !Self::handle_action(
                                &action.action,
                                id,
                                sender,
                                container,
                                visibilities,
                                request_action,
                            )
                        {
                            return false;
                        }
                    }
                }

                true
            }
        }
    }

    fn unhandle_action(
        action: &ActionType,
        id: usize,
        visibilities: &RwLock<HashMap<usize, Visibility>>,
    ) {
        if let ActionType::Style { action, .. } = &action {
            match action {
                StyleAction::SetVisibility { .. } => {
                    let contains = { visibilities.read().unwrap().contains_key(&id) };
                    if contains {
                        visibilities.write().unwrap().remove(&id);
                    }
                }
            }
        }
    }

    fn handle_style_action(
        action: &StyleAction,
        target: &ElementTarget,
        id: usize,
        container: &RwLock<ContainerElement>,
        visibilities: &RwLock<HashMap<usize, Visibility>>,
    ) -> bool {
        match action {
            StyleAction::SetVisibility(visibility) => {
                match target {
                    gigachad_actions::ElementTarget::StrId(str_id) => {
                        if let Some(element) = container
                            .read()
                            .unwrap()
                            .find_element_by_str_id(str_id)
                            .and_then(|x| x.container_element())
                        {
                            visibilities
                                .write()
                                .unwrap()
                                .insert(element.id, *visibility);
                        } else {
                            log::warn!("Could not find element with str id '{str_id}'");
                        }
                    }
                    gigachad_actions::ElementTarget::Id(id) => {
                        visibilities.write().unwrap().insert(*id, *visibility);
                    }
                    gigachad_actions::ElementTarget::SelfTarget => {
                        visibilities.write().unwrap().insert(id, *visibility);
                    }
                }

                true
            }
        }
    }

    #[allow(clippy::too_many_arguments, clippy::too_many_lines)]
    fn render_element(
        &self,
        ctx: &egui::Context,
        ui: &mut Ui,
        element: &Element,
        viewport: Option<&Viewport>,
        rect: Option<egui::Rect>,
        relative_container: Option<(egui::Rect, &ContainerElement)>,
        scroll_child: bool,
    ) {
        log::trace!("render_element: rect={rect:?}");

        if let Element::Table { .. } = element {
            self.render_table(ctx, ui, element, viewport, rect, relative_container);
            return;
        }

        if let Some(container) = element.container_element() {
            if scroll_child
                && self.handle_scroll_child_out_of_view(
                    ctx,
                    ui,
                    container,
                    viewport,
                    rect,
                    relative_container,
                )
            {
                return;
            }
        }

        let response = match element {
            Element::Input { input, .. } => Some(Self::render_input(ui, input, &self.checkboxes)),
            Element::Raw { value } => Some(ui.label(value)),
            Element::Image { source, element } => source
                .as_ref()
                .map(|source| self.render_image(ui, source, element)),
            Element::Canvas { element } => element
                .str_id
                .as_ref()
                .map_or_else(|| None, |str_id| self.render_canvas(ui, str_id, element)),
            _ => None,
        };

        if let Some(response) = response {
            self.handle_element_side_effects(
                ctx,
                Some(ui),
                element,
                viewport,
                rect,
                Some(&response),
                false,
            );
            return;
        }

        if let Some(container) = element.container_element() {
            if let Some(response) =
                self.render_container(ctx, ui, container, viewport, rect, relative_container)
            {
                if !self.container_hidden(container) {
                    self.handle_element_side_effects(
                        ctx,
                        Some(ui),
                        element,
                        viewport,
                        rect,
                        Some(&response),
                        false,
                    );
                }
            }
        }
    }

    fn render_input(
        ui: &mut Ui,
        input: &Input,
        checkboxes: &Arc<RwLock<HashMap<egui::Id, bool>>>,
    ) -> Response {
        match input {
            Input::Text { .. } | Input::Password { .. } => Self::render_text_input(ui, input),
            Input::Checkbox { .. } => Self::render_checkbox_input(ui, input, checkboxes),
        }
    }

    fn render_text_input(ui: &mut Ui, input: &Input) -> Response {
        let (Input::Text { value, .. } | Input::Password { value, .. }) = input else {
            unreachable!()
        };

        let id = ui.next_auto_id();
        let mut value_text = ui
            .data_mut(|data| data.remove_temp::<String>(id))
            .unwrap_or_else(|| value.clone().unwrap_or_default());
        let mut text_edit = egui::TextEdit::singleline(&mut value_text).id(id);

        if let Input::Password { .. } = input {
            text_edit = text_edit.password(true);
        }

        let response = text_edit.ui(ui);
        ui.data_mut(|data| data.insert_temp(id, value_text));
        response
    }

    fn render_checkbox_input(
        ui: &mut Ui,
        input: &Input,
        checkboxes: &Arc<RwLock<HashMap<egui::Id, bool>>>,
    ) -> Response {
        let Input::Checkbox { checked } = input else {
            unreachable!();
        };
        let checked = *checked;

        let id = ui.next_auto_id();

        let contains = { checkboxes.read().unwrap().contains_key(&id) };

        let mut checked_value = ui
            .data_mut(|data| {
                let value = data.remove_temp::<bool>(id);

                if !contains {
                    return None;
                }

                value
            })
            .unwrap_or_else(|| checked.unwrap_or_default());

        let checkbox = egui::Checkbox::without_text(&mut checked_value);
        let response = checkbox.ui(ui);

        ui.data_mut(|data| data.insert_temp(id, checked_value));

        if response.changed() {
            checkboxes.write().unwrap().insert(id, checked_value);
        }

        response
    }

    fn render_image(&self, ui: &mut Ui, source: &str, container: &ContainerElement) -> Response {
        egui::Frame::none()
            .show(ui, |ui| {
                ui.set_width(container.calculated_width.unwrap());
                ui.set_height(container.calculated_height.unwrap());

                let Some(AppImage::Bytes(bytes)) = self.images.read().unwrap().get(source).cloned()
                else {
                    return;
                };

                log::trace!(
                    "render_image: showing image for source={source} ({}, {})",
                    container.calculated_width.unwrap(),
                    container.calculated_height.unwrap(),
                );

                egui::Image::from_bytes(source.to_string(), egui::load::Bytes::Shared(bytes))
                    .max_width(container.calculated_width.unwrap())
                    .max_height(container.calculated_height.unwrap())
                    .ui(ui);
            })
            .response
    }

    fn render_canvas(
        &self,
        ui: &mut Ui,
        str_id: &str,
        container: &ContainerElement,
    ) -> Option<Response> {
        self.canvas_actions.read().unwrap().get(str_id).map_or_else(
            || None,
            |actions| {
                let (response, painter) = ui.allocate_painter(
                    egui::Vec2::new(
                        container.calculated_width.unwrap(),
                        container.calculated_height.unwrap(),
                    ),
                    egui::Sense::hover(),
                );

                let pixels_per_point = 1.0; // ctx.pixels_per_point();
                let cursor_px = egui::Pos2::new(
                    response.rect.min.x * pixels_per_point,
                    response.rect.min.y * pixels_per_point,
                )
                .ceil();

                let default_color = Color32::BLACK;
                let stroke = &mut egui::epaint::PathStroke::new(1.0, default_color).inside();
                stroke.color = egui::epaint::ColorMode::Solid(default_color);

                for action in actions {
                    match action {
                        CanvasAction::Clear => {}
                        CanvasAction::StrokeSize(size) => {
                            stroke.width = *size;
                        }
                        CanvasAction::StrokeColor(color) => {
                            stroke.color = egui::epaint::ColorMode::Solid((*color).into());
                        }
                        CanvasAction::Line(start, end) => {
                            painter.line_segment(
                                [
                                    egui::Pos2::new(start.0 + cursor_px.x, start.1 + cursor_px.y),
                                    egui::Pos2::new(end.0 + cursor_px.x, end.1 + cursor_px.y),
                                ],
                                stroke.clone(),
                            );
                        }
                        CanvasAction::FillRect(start, end) => {
                            let egui::epaint::ColorMode::Solid(color) = stroke.color else {
                                continue;
                            };
                            painter.rect_filled(
                                egui::Rect::from_min_max(
                                    egui::Pos2::new(start.0 + cursor_px.x, start.1 + cursor_px.y),
                                    egui::Pos2::new(end.0 + cursor_px.x, end.1 + cursor_px.y),
                                ),
                                0.0,
                                color,
                            );
                        }
                    }
                }

                Some(response)
            },
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn render_table(
        &self,
        ctx: &egui::Context,
        ui: &mut Ui,
        element: &Element,
        viewport: Option<&Viewport>,
        rect: Option<egui::Rect>,
        relative_container: Option<(egui::Rect, &ContainerElement)>,
    ) {
        let TableIter { rows, headings } = element.table_iter();

        let grid = egui::Grid::new(format!("grid-{}", element.container_element().unwrap().id));

        grid.show(ui, |ui| {
            if let Some(headings) = headings {
                for heading in headings {
                    for th in heading {
                        egui::Frame::none().show(ui, |ui| {
                            self.render_container(ctx, ui, th, viewport, rect, relative_container);
                        });
                    }
                    ui.end_row();
                }
            }
            for row in rows {
                for td in row {
                    egui::Frame::none().show(ui, |ui| {
                        self.render_container(ctx, ui, td, viewport, rect, relative_container);
                    });
                }
                ui.end_row();
            }
        });
    }

    fn trigger_side_effect(&self, handler: impl Fn() -> bool + Send + Sync + 'static) {
        self.side_effects
            .lock()
            .unwrap()
            .push_back(Box::new(handler));
    }

    fn paint(&self, ctx: &egui::Context) {
        self.calc(ctx);

        let container = self.container.clone();
        let container: &ContainerElement = &container.read().unwrap();

        ctx.memory_mut(|x| {
            x.options.line_scroll_speed = 100.0;
        });

        ctx.style_mut(|style| {
            style.spacing.window_margin.left = 0.0;
            style.spacing.window_margin.right = 0.0;
            style.spacing.window_margin.top = 0.0;
            style.spacing.window_margin.bottom = 0.0;
            style.spacing.item_spacing = egui::emath::Vec2::splat(0.0);
            #[cfg(all(debug_assertions, feature = "debug"))]
            {
                style.debug.debug_on_hover = true;
            }
        });

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                egui::Frame::none()
                    .fill(
                        self.background
                            .unwrap_or_else(|| Color32::from_hex("#181a1b").unwrap()),
                    )
                    .show(ui, |ui| {
                        self.render_container(ctx, ui, container, None, None, None);
                    });
            });

        let mut handlers_count = 0;

        for handler in self.side_effects.lock().unwrap().drain(..) {
            handlers_count += 1;
            if !handler() {
                break;
            }
        }

        log::trace!("paint: {handlers_count} handler(s) on render");
    }
}

impl eframe::App for EguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.paint(ctx);
    }
}

const fn cursor_to_cursor_icon(cursor: Cursor) -> CursorIcon {
    match cursor {
        Cursor::Auto => CursorIcon::Default,
        Cursor::Pointer => CursorIcon::PointingHand,
        Cursor::Text => CursorIcon::Text,
        Cursor::Crosshair => CursorIcon::Crosshair,
        Cursor::Move => CursorIcon::Move,
        Cursor::NotAllowed => CursorIcon::NotAllowed,
        Cursor::NoDrop => CursorIcon::NoDrop,
        Cursor::Grab => CursorIcon::Grab,
        Cursor::Grabbing => CursorIcon::Grabbing,
        Cursor::AllScroll => CursorIcon::AllScroll,
        Cursor::ColResize => CursorIcon::ResizeColumn,
        Cursor::RowResize => CursorIcon::ResizeRow,
        Cursor::NResize => CursorIcon::ResizeNorth,
        Cursor::EResize => CursorIcon::ResizeEast,
        Cursor::SResize => CursorIcon::ResizeSouth,
        Cursor::WResize => CursorIcon::ResizeWest,
        Cursor::NeResize => CursorIcon::ResizeNorthEast,
        Cursor::NwResize => CursorIcon::ResizeNorthWest,
        Cursor::SeResize => CursorIcon::ResizeSouthEast,
        Cursor::SwResize => CursorIcon::ResizeSouthWest,
        Cursor::EwResize => CursorIcon::ResizeHorizontal,
        Cursor::NsResize => CursorIcon::ResizeVertical,
        Cursor::NeswResize => CursorIcon::ResizeNwSe,
        Cursor::ZoomIn => CursorIcon::ZoomIn,
        Cursor::ZoomOut => CursorIcon::ZoomOut,
    }
}
