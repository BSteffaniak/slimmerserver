#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

use std::{collections::HashMap, io::Write};

use async_trait::async_trait;
use const_format::concatcp;
use hyperchad_renderer::{canvas, Color, HtmlTagRenderer, PartialView, View};
use hyperchad_renderer_html::{
    extend::{ExtendHtmlRenderer, HtmlRendererEventPub},
    html::write_attr,
    DefaultHtmlTagRenderer,
};
use hyperchad_transformer::{models::Route, Container, ResponsiveTrigger};
use maud::{html, PreEscaped, DOCTYPE};

#[derive(Default, Clone)]
pub struct VanillaJsTagRenderer {
    default: DefaultHtmlTagRenderer,
}

const SCRIPT_NAME_STEM: &str = "hyperchad";
#[cfg(debug_assertions)]
const SCRIPT_NAME_EXTENSION: &str = "js";
#[cfg(not(debug_assertions))]
const SCRIPT_NAME_EXTENSION: &str = "min.js";

pub const SCRIPT_NAME: &str = concatcp!(SCRIPT_NAME_STEM, ".", SCRIPT_NAME_EXTENSION);

#[cfg(all(debug_assertions, feature = "script"))]
pub const SCRIPT: &str = include_str!("../web/dist/index.js");

#[cfg(all(not(debug_assertions), feature = "script"))]
pub const SCRIPT: &str = include_str!("../web/dist/index.min.js");

#[cfg(all(feature = "hash", feature = "script"))]
pub static SCRIPT_NAME_HASHED: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    let digest = md5::compute(SCRIPT.as_bytes());
    let digest = format!("{digest:x}");
    let hash = &digest[..10];
    format!("{SCRIPT_NAME_STEM}-{hash}.{SCRIPT_NAME_EXTENSION}")
});

impl HtmlTagRenderer for VanillaJsTagRenderer {
    fn add_responsive_trigger(&mut self, name: String, trigger: ResponsiveTrigger) {
        self.default.responsive_triggers.insert(name, trigger);
    }

    fn element_attrs_to_html(
        &self,
        f: &mut dyn Write,
        container: &Container,
        is_flex_child: bool,
    ) -> Result<(), std::io::Error> {
        self.default
            .element_attrs_to_html(f, container, is_flex_child)?;

        if let Some(route) = &container.route {
            match route {
                Route::Get {
                    route,
                    trigger,
                    swap,
                } => {
                    match swap {
                        hyperchad_transformer::models::SwapTarget::This => {
                            write_attr(f, b"hx-swap", b"outerHTML")?;
                        }
                        hyperchad_transformer::models::SwapTarget::Children => {
                            write_attr(f, b"hx-swap", b"innerHTML")?;
                        }
                    }
                    write_attr(f, b"hx-get", route.as_bytes())?;
                    if let Some(trigger) = trigger {
                        write_attr(f, b"hx-trigger", trigger.as_bytes())?;
                    }
                }
                Route::Post {
                    route,
                    trigger,
                    swap,
                } => {
                    match swap {
                        hyperchad_transformer::models::SwapTarget::This => {
                            write_attr(f, b"hx-swap", b"outerHTML")?;
                        }
                        hyperchad_transformer::models::SwapTarget::Children => {
                            write_attr(f, b"hx-swap", b"innerHTML")?;
                        }
                    }
                    write_attr(f, b"hx-swap", b"outerHTML")?;
                    write_attr(f, b"hx-post", route.as_bytes())?;
                    if let Some(trigger) = trigger {
                        write_attr(f, b"hx-trigger", trigger.as_bytes())?;
                    }
                }
            }
        }

        Ok(())
    }

    fn partial_html(
        &self,
        _headers: &HashMap<String, String>,
        container: &Container,
        content: String,
        _viewport: Option<&str>,
        _background: Option<Color>,
    ) -> String {
        let mut responsive_css = vec![];
        self.default
            .reactive_conditions_to_css(&mut responsive_css, container)
            .unwrap();
        let responsive_css = std::str::from_utf8(&responsive_css).unwrap();

        format!("{responsive_css}\n\n{content}")
    }

    fn root_html(
        &self,
        _headers: &HashMap<String, String>,
        container: &Container,
        content: String,
        viewport: Option<&str>,
        background: Option<Color>,
        title: Option<&str>,
        description: Option<&str>,
    ) -> String {
        let mut responsive_css = vec![];
        self.default
            .reactive_conditions_to_css(&mut responsive_css, container)
            .unwrap();
        let responsive_css = std::str::from_utf8(&responsive_css).unwrap();

        let background = background.map(|x| format!("background:rgb({},{},{})", x.r, x.g, x.b));
        let background = background.as_deref().unwrap_or("");

        #[cfg(all(feature = "hash", feature = "script"))]
        let script = html! { script src={"/js/"(SCRIPT_NAME_HASHED.as_str())} {} };
        #[cfg(not(all(feature = "hash", feature = "script")))]
        let script = html! { script src={"/js/"(SCRIPT_NAME)} {} };

        html! {
            (DOCTYPE)
            html style="height:100%" lang="en" {
                head {
                    @if let Some(title) = title {
                        title { (title) }
                    }
                    @if let Some(description) = description {
                        meta name="description" content=(description);
                    }
                    style {(format!(r"
                        body {{
                            margin: 0;{background};
                            overflow: hidden;
                        }}
                        .remove-button-styles {{
                            background: none;
                            color: inherit;
                            border: none;
                            padding: 0;
                            font: inherit;
                            cursor: pointer;
                            outline: inherit;
                        }}
                    "))}
                    (script)
                    (PreEscaped(responsive_css))
                    @if let Some(content) = viewport {
                        meta name="viewport" content=(content);
                    }
                }
                body style="height:100%" {
                    (PreEscaped(content))
                }
            }
        }
        .into_string()
    }
}

pub struct VanillaJsRenderer {}

#[async_trait]
impl ExtendHtmlRenderer for VanillaJsRenderer {
    /// # Errors
    ///
    /// Will error if `VanillaJsRenderer` fails to emit the event.
    async fn emit_event(
        &self,
        _publisher: HtmlRendererEventPub,
        _event_name: String,
        _event_value: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + 'static>> {
        Ok(())
    }

    /// # Errors
    ///
    /// Will error if `VanillaJsRenderer` fails to render the view.
    async fn render(
        &self,
        _publisher: HtmlRendererEventPub,
        _view: View,
    ) -> Result<(), Box<dyn std::error::Error + Send + 'static>> {
        Ok(())
    }

    /// # Errors
    ///
    /// Will error if `VanillaJsRenderer` fails to render the partial elements.
    async fn render_partial(
        &self,
        _publisher: HtmlRendererEventPub,
        _partial: PartialView,
    ) -> Result<(), Box<dyn std::error::Error + Send + 'static>> {
        log::warn!("render_partial: partial={_partial:?}");
        Ok(())
    }

    /// # Errors
    ///
    /// Will error if `VanillaJsRenderer` fails to render the canvas update.
    async fn render_canvas(
        &self,
        _publisher: HtmlRendererEventPub,
        _update: canvas::CanvasUpdate,
    ) -> Result<(), Box<dyn std::error::Error + Send + 'static>> {
        Ok(())
    }
}
