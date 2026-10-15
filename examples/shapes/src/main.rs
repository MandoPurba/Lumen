use kireiui::{Color, DrawCommand};

fn main() {
    kireiui::run(|width, height| {
        let mut cmds = Vec::with_capacity(1000);
        for i in 0..1000 {
            let f = i as f32;
            let x = (f * 137.5 + 43.0) % width;
            let y = (f * 97.3 + 71.0) % height;
            let w = 20.0 + (f * 23.7 % 60.0);
            let h = 20.0 + (f * 31.3 % 60.0);
            let r = f * 7.3 % 16.0;
            let (cr, cg, cb) = hue_to_rgb(f * 0.618_034 % 1.0);
            cmds.push(DrawCommand::Rect {
                position: [x, y],
                size: [w, h],
                color: Color::rgba(cr, cg, cb, 0.8),
                corner_radius: r,
                border_width: 1.5,
                border_color: Color::rgba(cr * 0.5, cg * 0.5, cb * 0.5, 1.0),
            });
        }
        cmds
    });
}

fn hue_to_rgb(hue: f32) -> (f32, f32, f32) {
    let h = hue * 6.0;
    let c = 0.8_f32;
    let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
    let (r, g, b) = match h as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (r + 0.15, g + 0.15, b + 0.15)
}
