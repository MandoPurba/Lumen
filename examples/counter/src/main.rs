use lumen::reactive::{create_effect, create_signal, get, update};
use lumen::{run_reactive, Color, DrawCommand, Event};

fn main() {
    let count = create_signal(0_i32);

    create_effect(move || {
        let c = get(count);
        eprintln!("[counter] count = {c}");
    });

    run_reactive(move |w, h, text_ctx, events| {
        let btn_w = 120.0_f32;
        let btn_h = 44.0_f32;
        let center_x = w / 2.0;
        let btn_y = h / 2.0 + 20.0;
        let inc_x = center_x + 10.0;
        let dec_x = center_x - 70.0;

        for event in events {
            if let Event::PointerPressed { x, y } = event {
                if *y >= btn_y && *y <= btn_y + btn_h {
                    if *x >= inc_x && *x <= inc_x + btn_w {
                        update(count, |c| c + 1);
                    } else if *x >= dec_x && *x <= dec_x + 60.0 {
                        update(count, |c| c - 1);
                    }
                }
            }
        }

        let c = get(count);

        let label = format!("Count: {c}");
        let label_buf = text_ctx.create_buffer(&label, 32.0, 40.0, None);
        let inc_buf = text_ctx.create_buffer("+1", 20.0, 28.0, None);
        let dec_buf = text_ctx.create_buffer("\u{2212}1", 20.0, 28.0, None);

        vec![
            DrawCommand::Rect {
                position: [0.0, 0.0],
                size: [w, h],
                color: Color::rgba(0.08, 0.08, 0.12, 1.0),
                corner_radius: 0.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
            text_ctx.draw_text(&label_buf, center_x - 60.0, h / 2.0 - 40.0, Color::WHITE),
            DrawCommand::Rect {
                position: [inc_x, btn_y],
                size: [btn_w, btn_h],
                color: Color::rgba(0.2, 0.5, 0.8, 1.0),
                corner_radius: 8.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
            text_ctx.draw_text(
                &inc_buf,
                inc_x + btn_w / 2.0 - 10.0,
                btn_y + 10.0,
                Color::WHITE,
            ),
            DrawCommand::Rect {
                position: [dec_x, btn_y],
                size: [60.0, btn_h],
                color: Color::rgba(0.6, 0.2, 0.2, 1.0),
                corner_radius: 8.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
            text_ctx.draw_text(&dec_buf, dec_x + 18.0, btn_y + 10.0, Color::WHITE),
        ]
    });
}
