# Lumen

Declarative, cross-platform UI framework in pure Rust.

SwiftUI/Compose-style API. Own GPU renderer (wgpu). Own design system. Built-in reactive data layer. No HTML/CSS.

One codebase &rarr; desktop, web, mobile. Pixel-consistent across all platforms.

## Features

- **Declarative UI** — Build interfaces with composable views and modifier chains
- **GPU-rendered** — wgpu backend with SDF rounded rects, instanced drawing, glyph atlas
- **Fine-grained reactivity** — Arena-owned signals, memos, effects with automatic dependency tracking
- **Built-in design system** — Semantic tokens, light/dark themes, 10 styled components
- **Persistent data layer** — `#[derive(Model)]`, `Handle<T>`, reactive queries, SQLite (desktop), in-memory (web)
- **Accessibility** — AccessKit integration for screen readers on desktop
- **Cross-platform** — Desktop (macOS/Windows/Linux), web (WebGPU/WebGL2), mobile (Android/iOS)
- **Animation** — Springs, transitions, easing curves, reduce-motion support

## Quick start

Add to your `Cargo.toml`:

```toml
[dependencies]
lumen = "0.1"
```

A minimal counter app:

```rust
use lumen::reactive::{create_signal, get, set};
use lumen::{hit_test, run_app, AppFrame, Color, DrawCommand, Event, TextDraw};
use lumen::a11y::A11yTree;

fn main() {
    let count = create_signal(0i32);

    run_app(move |w, h, text_ctx, events| {
        for ev in events {
            if let Event::PointerPressed { x, y } = ev {
                if hit_test(*x, *y, w / 2.0 - 50.0, h / 2.0, 100.0, 40.0) {
                    set(count, get(count) + 1);
                }
            }
        }

        let label = format!("Count: {}", get(count));
        let mut cmds = vec![
            DrawCommand::Rect {
                position: [0.0, 0.0],
                size: [w, h],
                color: Color::new(0.06, 0.06, 0.1, 1.0),
                corner_radius: 0.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
            DrawCommand::Rect {
                position: [w / 2.0 - 50.0, h / 2.0],
                size: [100.0, 40.0],
                color: Color::new(0.3, 0.5, 1.0, 1.0),
                corner_radius: 8.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
        ];
        cmds.push(text_ctx.draw_label(&label, 18.0, w / 2.0 - 40.0, h / 2.0 - 30.0, Color::WHITE));

        AppFrame { commands: cmds, a11y: A11yTree::empty("Counter") }
    });
}
```

## Examples

| Example | Description |
|---------|-------------|
| `shapes` | 1,000 rounded rects at 60fps — GPU renderer stress test |
| `text` | Mixed Latin/CJK/emoji text with line wrapping |
| `counter` | Reactive signal-driven counter |
| `todo` | In-memory todo app with keyboard + accessibility |
| `gallery` | Design system component gallery (all 10 widgets) |
| `todo-db` | Persistent todo app with SQLite backend |
| `todo-mobile` | Mobile todo app with touch input (Android/iOS) |

Run any example:

```sh
cargo run -p counter
cargo run -p gallery
cargo run -p todo-db
```

## Architecture

```
lumen              facade crate (re-exports everything)
lumen-widgets      styled components (buttons, text fields, checkboxes, ...)
lumen-theme        design tokens, light/dark themes, contrast validation
lumen-primitives   headless views (VStack, HStack, ZStack, SizedBox)
lumen-data         Model store, Handle<T>, reactive queries, persistence
lumen-core         View trait, reactive runtime, events, animation, a11y, focus
lumen-layout       constraint-based layout engine
lumen-text         text shaping/rasterization (cosmic-text)
lumen-render       display list → wgpu GPU renderer
lumen-platform     platform shells (desktop, web, mobile)
```

Users depend only on the `lumen` facade crate.

## Building

```sh
cargo build --workspace
cargo test --workspace
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
```

### Web (wasm32)

Requires [trunk](https://trunkrs.dev/):

```sh
cd examples/todo-db
trunk serve
```

### Mobile

**Android** (requires NDK + [cargo-ndk](https://github.com/nicepkg/cargo-ndk)):

```sh
cargo ndk -t arm64-v8a build -p todo-mobile
```

**iOS**: Build as a static library and link into an Xcode project. winit handles UIKit integration.

## Platform support

| Platform | Renderer | Text | Data | A11y |
|----------|----------|------|------|------|
| Desktop (macOS/Win/Linux) | wgpu (Vulkan/Metal/DX12) | cosmic-text | SQLite | AccessKit |
| Web | wgpu (WebGPU/WebGL2) | Canvas 2D | In-memory | Planned |
| Android | wgpu (Vulkan/GLES) | cosmic-text | SQLite | Planned |
| iOS | wgpu (Metal) | cosmic-text | SQLite | Planned |

## MSRV

Rust 1.97 (stable &minus; 2).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
