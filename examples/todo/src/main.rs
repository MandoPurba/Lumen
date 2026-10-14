use lumen::a11y::{A11yNode, A11yRole, A11yTree};
use lumen::focus::FocusManager;
use lumen::reactive::{create_signal, get, update};
use lumen::text_input::TextEditState;
use lumen::{hit_test, run_app, AppFrame, Color, DrawCommand, Event, Key};

#[derive(Clone)]
struct TodoItem {
    #[allow(dead_code)]
    id: u32,
    text: String,
    done: bool,
}

const BG: Color = Color::rgba(0.08, 0.08, 0.12, 1.0);
const CARD_BG: Color = Color::rgba(0.14, 0.14, 0.20, 1.0);
const INPUT_BG: Color = Color::rgba(0.10, 0.10, 0.16, 1.0);
const ACCENT: Color = Color::rgba(0.30, 0.55, 0.85, 1.0);
const ACCENT_HOVER: Color = Color::rgba(0.35, 0.60, 0.90, 1.0);
const DONE_COLOR: Color = Color::rgba(0.45, 0.75, 0.45, 1.0);
const DELETE_COLOR: Color = Color::rgba(0.75, 0.30, 0.30, 1.0);
const FOCUS_RING: Color = Color::rgba(0.40, 0.65, 1.0, 0.8);
const TEXT_DIM: Color = Color::rgba(0.55, 0.55, 0.60, 1.0);

fn main() {
    let todos = create_signal(Vec::<TodoItem>::new());
    let input = TextEditState::new();
    let focus = FocusManager::new();
    let next_id = create_signal(1_u32);
    let scroll_y = create_signal(0.0_f32);

    run_app(move |w, h, text_ctx, events| {
        let todo_list = get(todos);
        let todo_count = todo_list.len();

        // Focus layout: 0=input, 1=add button, 2..2+N = todo toggle, 2+N..2+2N = delete
        let focus_count = 2 + todo_count * 2;

        // --- Event processing ---
        for event in events {
            match event {
                Event::KeyPressed {
                    key: Key::Tab,
                    modifiers,
                } => {
                    focus.handle_tab(focus_count, modifiers.shift);
                }

                Event::KeyPressed { key, modifiers, .. } if focus.is_focused(0) => match key {
                    Key::Enter => {
                        add_todo(&input, todos, next_id);
                    }
                    Key::Char(ch) => {
                        if modifiers.meta && *ch == 'a' {
                            input.select_all();
                        } else {
                            input.insert(*ch);
                        }
                    }
                    Key::Backspace => input.delete_backward(),
                    Key::Delete => input.delete_forward(),
                    Key::ArrowLeft => input.move_left(modifiers.shift),
                    Key::ArrowRight => input.move_right(modifiers.shift),
                    Key::Home => input.move_home(modifiers.shift),
                    Key::End => input.move_end(modifiers.shift),
                    _ => {}
                },

                Event::KeyPressed {
                    key: Key::Enter | Key::Space,
                    ..
                } if focus.is_focused(1) => {
                    add_todo(&input, todos, next_id);
                }

                Event::KeyPressed {
                    key: Key::Enter | Key::Space,
                    ..
                } => {
                    let fi = focus.focused();
                    if let Some(idx) = fi {
                        if idx >= 2 && idx < 2 + todo_count {
                            let ti = idx - 2;
                            update(todos, |mut t| {
                                t[ti].done = !t[ti].done;
                                t
                            });
                        } else if idx >= 2 + todo_count && idx < 2 + 2 * todo_count {
                            let ti = idx - 2 - todo_count;
                            update(todos, |mut t| {
                                t.remove(ti);
                                t
                            });
                        }
                    }
                }

                _ => {}
            }
        }

        // Mouse clicks
        let todo_list = get(todos);
        let todo_count = todo_list.len();

        let pad = 24.0_f32;
        let content_w = (w - pad * 2.0).min(480.0);
        let start_x = (w - content_w) / 2.0;

        let input_y = 80.0;
        let input_h = 40.0;
        let btn_w = 72.0;
        let input_w = content_w - btn_w - 8.0;

        let list_top = input_y + input_h + 16.0;
        let item_h = 44.0;
        let item_gap = 4.0;
        let scroll = get(scroll_y);

        for event in events {
            match event {
                Event::PointerPressed { x, y } => {
                    if hit_test(*x, *y, start_x + input_w + 8.0, input_y, btn_w, input_h) {
                        add_todo(&input, todos, next_id);
                        focus.focus(0);
                    } else if hit_test(*x, *y, start_x, input_y, input_w, input_h) {
                        focus.focus(0);
                    } else {
                        for i in 0..todo_count {
                            let iy = list_top + i as f32 * (item_h + item_gap) - scroll;
                            if iy + item_h < list_top || iy > h {
                                continue;
                            }
                            let check_x = start_x + 8.0;
                            let check_w = 28.0;
                            let del_x = start_x + content_w - 36.0;
                            let del_w = 28.0;

                            if hit_test(*x, *y, check_x, iy + 8.0, check_w, check_w) {
                                update(todos, |mut t| {
                                    t[i].done = !t[i].done;
                                    t
                                });
                                break;
                            } else if hit_test(*x, *y, del_x, iy + 8.0, del_w, del_w) {
                                update(todos, |mut t| {
                                    t.remove(i);
                                    t
                                });
                                break;
                            }
                        }
                    }
                }
                Event::Scroll { dy, .. } => {
                    let max_scroll =
                        (todo_count as f32 * (item_h + item_gap) - (h - list_top)).max(0.0);
                    update(scroll_y, |s| (s - dy).clamp(0.0, max_scroll));
                }
                _ => {}
            }
        }

        // Re-read after possible mutations
        let todo_list = get(todos);
        let todo_count = todo_list.len();
        let scroll = get(scroll_y);

        // --- Rendering ---
        let mut cmds = Vec::with_capacity(16 + todo_count * 6);

        // Background
        cmds.push(DrawCommand::Rect {
            position: [0.0, 0.0],
            size: [w, h],
            color: BG,
            corner_radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        });

        // Title
        let title_buf = text_ctx.create_buffer("Todo", 28.0, 36.0, None);
        cmds.push(text_ctx.draw_text(&title_buf, start_x, 28.0, Color::WHITE));

        // Input field background
        let input_focused = focus.is_focused(0);
        cmds.push(DrawCommand::Rect {
            position: [start_x, input_y],
            size: [input_w, input_h],
            color: INPUT_BG,
            corner_radius: 8.0,
            border_width: if input_focused { 2.0 } else { 1.0 },
            border_color: if input_focused {
                FOCUS_RING
            } else {
                Color::rgba(0.25, 0.25, 0.30, 1.0)
            },
        });

        // Input text + cursor
        let input_text = input.text();
        if input_text.is_empty() && !input_focused {
            let ph = text_ctx.create_buffer("What needs to be done?", 16.0, 22.0, None);
            cmds.push(text_ctx.draw_text(&ph, start_x + 12.0, input_y + 10.0, TEXT_DIM));
        } else {
            let txt_buf = text_ctx.create_buffer(&input_text, 16.0, 22.0, None);
            cmds.push(text_ctx.draw_text(&txt_buf, start_x + 12.0, input_y + 10.0, Color::WHITE));

            if input_focused {
                let cursor_idx = input.cursor_char_index();
                let cursor_x = start_x + 12.0 + cursor_idx as f32 * 9.6;
                cmds.push(DrawCommand::Rect {
                    position: [cursor_x, input_y + 8.0],
                    size: [2.0, 24.0],
                    color: FOCUS_RING,
                    corner_radius: 1.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                });
            }
        }

        // Add button
        let add_focused = focus.is_focused(1);
        cmds.push(DrawCommand::Rect {
            position: [start_x + input_w + 8.0, input_y],
            size: [btn_w, input_h],
            color: if add_focused { ACCENT_HOVER } else { ACCENT },
            corner_radius: 8.0,
            border_width: if add_focused { 2.0 } else { 0.0 },
            border_color: FOCUS_RING,
        });
        let add_buf = text_ctx.create_buffer("Add", 16.0, 22.0, None);
        cmds.push(text_ctx.draw_text(
            &add_buf,
            start_x + input_w + 8.0 + btn_w / 2.0 - 14.0,
            input_y + 10.0,
            Color::WHITE,
        ));

        // Todo list items
        for (i, todo) in todo_list.iter().enumerate() {
            let iy = list_top + i as f32 * (item_h + item_gap) - scroll;
            if iy + item_h < list_top || iy > h {
                continue;
            }

            let toggle_focus_idx = 2 + i;
            let delete_focus_idx = 2 + todo_count + i;
            let toggle_focused = focus.is_focused(toggle_focus_idx);
            let delete_focused = focus.is_focused(delete_focus_idx);

            // Item background
            cmds.push(DrawCommand::Rect {
                position: [start_x, iy],
                size: [content_w, item_h],
                color: CARD_BG,
                corner_radius: 8.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            });

            // Checkbox
            let check_x = start_x + 8.0;
            let check_y = iy + 8.0;
            let check_size = 28.0;
            cmds.push(DrawCommand::Rect {
                position: [check_x, check_y],
                size: [check_size, check_size],
                color: if todo.done {
                    DONE_COLOR
                } else {
                    Color::rgba(0.2, 0.2, 0.25, 1.0)
                },
                corner_radius: 6.0,
                border_width: if toggle_focused { 2.0 } else { 1.0 },
                border_color: if toggle_focused {
                    FOCUS_RING
                } else {
                    Color::rgba(0.3, 0.3, 0.35, 1.0)
                },
            });

            if todo.done {
                let check_buf = text_ctx.create_buffer("\u{2713}", 18.0, 24.0, None);
                cmds.push(text_ctx.draw_text(
                    &check_buf,
                    check_x + 6.0,
                    check_y + 2.0,
                    Color::WHITE,
                ));
            }

            // Todo text
            let text_color = if todo.done { TEXT_DIM } else { Color::WHITE };
            let todo_buf = text_ctx.create_buffer(&todo.text, 16.0, 22.0, None);
            cmds.push(text_ctx.draw_text(&todo_buf, start_x + 44.0, iy + 12.0, text_color));

            // Delete button
            let del_x = start_x + content_w - 36.0;
            let del_y = iy + 8.0;
            cmds.push(DrawCommand::Rect {
                position: [del_x, del_y],
                size: [28.0, 28.0],
                color: if delete_focused {
                    DELETE_COLOR
                } else {
                    Color::rgba(0.25, 0.15, 0.15, 1.0)
                },
                corner_radius: 6.0,
                border_width: if delete_focused { 2.0 } else { 0.0 },
                border_color: FOCUS_RING,
            });
            let del_buf = text_ctx.create_buffer("\u{00D7}", 18.0, 24.0, None);
            cmds.push(text_ctx.draw_text(&del_buf, del_x + 7.0, del_y + 2.0, Color::WHITE));
        }

        // Item count
        let count_text = match todo_count {
            0 => "No items".to_string(),
            1 => "1 item".to_string(),
            n => format!("{n} items"),
        };
        let count_buf = text_ctx.create_buffer(&count_text, 13.0, 18.0, None);
        let count_y = if todo_count == 0 {
            list_top + 8.0
        } else {
            list_top + todo_count as f32 * (item_h + item_gap) - scroll + 8.0
        };
        cmds.push(text_ctx.draw_text(&count_buf, start_x, count_y, TEXT_DIM));

        // --- Accessibility tree ---
        let a11y = build_a11y(&todo_list, &input, &focus);

        AppFrame {
            commands: cmds,
            a11y,
        }
    });
}

fn add_todo(
    input: &TextEditState,
    todos: lumen::reactive::Signal<Vec<TodoItem>>,
    next_id: lumen::reactive::Signal<u32>,
) {
    let text = input.text();
    let trimmed = text.trim().to_string();
    if trimmed.is_empty() {
        return;
    }
    let id = get(next_id);
    update(todos, |mut t| {
        t.push(TodoItem {
            id,
            text: trimmed,
            done: false,
        });
        t
    });
    update(next_id, |n| n + 1);
    input.clear();
}

fn build_a11y(todos: &[TodoItem], input: &TextEditState, focus: &FocusManager) -> A11yTree {
    let todo_count = todos.len();
    let mut nodes = Vec::with_capacity(4 + todo_count * 3);

    // Root window
    let mut root_children = vec![10, 11];
    if !todos.is_empty() {
        root_children.push(20);
    }
    nodes.push(
        A11yNode::new(1, A11yRole::Window)
            .name("Todo")
            .children(root_children),
    );

    // Text input (id=10)
    let mut input_node = A11yNode::new(10, A11yRole::TextField)
        .name("Add todo")
        .value(&input.text())
        .focusable();
    if focus.is_focused(0) {
        // Already focusable; focus tracked via tree.focus
    }
    let _ = &mut input_node; // suppress unused
    nodes.push(input_node);

    // Add button (id=11)
    nodes.push(A11yNode::new(11, A11yRole::Button).name("Add").focusable());

    // List
    if !todos.is_empty() {
        let list_children: Vec<u64> = (0..todo_count).map(|i| 100 + i as u64 * 10).collect();
        nodes.push(
            A11yNode::new(20, A11yRole::List)
                .name("Todo list")
                .children(list_children),
        );

        for (i, todo) in todos.iter().enumerate() {
            let item_base = 100 + i as u64 * 10;
            let check_id = item_base + 1;
            let del_id = item_base + 2;

            nodes.push(
                A11yNode::new(item_base, A11yRole::ListItem)
                    .name(&todo.text)
                    .children(vec![check_id, del_id]),
            );

            let label = if todo.done {
                format!("{}, completed", todo.text)
            } else {
                format!("{}, not completed", todo.text)
            };
            nodes.push(
                A11yNode::new(check_id, A11yRole::CheckBox)
                    .name(&label)
                    .checked(todo.done)
                    .focusable(),
            );

            nodes.push(
                A11yNode::new(del_id, A11yRole::Button)
                    .name(&format!("Delete {}", todo.text))
                    .focusable(),
            );
        }
    }

    // Determine focused a11y ID
    let focus_a11y_id = focus.focused().map(|fi| {
        if fi == 0 {
            10_u64
        } else if fi == 1 {
            11
        } else if fi < 2 + todo_count {
            100 + (fi - 2) as u64 * 10 + 1 // checkbox
        } else {
            100 + (fi - 2 - todo_count) as u64 * 10 + 2 // delete button
        }
    });

    A11yTree {
        root: 1,
        nodes,
        focus: focus_a11y_id,
    }
}
