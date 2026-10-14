use lumen_core::Color;

/// A draw command in the display list.
pub enum DrawCommand {
    /// Filled rectangle with optional rounded corners and border.
    Rect {
        position: [f32; 2],
        size: [f32; 2],
        color: Color,
        corner_radius: f32,
        border_width: f32,
        border_color: Color,
    },
}
