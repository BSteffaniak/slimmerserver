#![allow(clippy::module_name_repetitions)]

use hyperchad_color::Color;

#[derive(Debug, Clone, Copy)]
pub struct Pos(pub f32, pub f32);

#[derive(Debug, Clone)]
pub enum CanvasAction {
    StrokeSize(f32),
    StrokeColor(Color),
    Line(Pos, Pos),
    FillRect(Pos, Pos),
    Clear,
}

#[derive(Default, Debug, Clone)]
pub struct CanvasUpdate {
    pub target: String,
    pub canvas_actions: Vec<CanvasAction>,
}
