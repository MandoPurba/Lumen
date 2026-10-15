# Roadmap — Sihora

Each milestone has exit criteria. Don't start next until met.

## M0 — Bootstrap
- Cargo workspace, crates stubbed (`core`, `render`, `platform`, facade).
- CI: fmt, clippy, test on macOS/Windows/Linux + `wasm32-unknown-unknown` compile check.
- License, README, docs/.
- **Exit:** CI green on empty crates.

## M1 — Pixels
- winit window + wgpu surface, clear color, resize, HiDPI.
- Draw rects, rounded rects (SDF), borders, solid colors.
- **Exit:** `examples/shapes` draws 1,000 rounded rects at 60fps+.

## M2 — Text ✓
- Font loading, shaping, glyph atlas, wrapping.
- **Exit:** paragraph with mixed Latin/CJK/emoji renders correctly; resizes reflow.

## M3 — Layout + View tree ✓
- `View` trait, arena tree, `VStack/HStack/ZStack`, padding, frame, alignment, spacing.
- **Exit:** static layout example matches expected bounds (unit tests).

## M4 — Reactivity + Events ✓
- `Signal/Memo/Effect`, owner scopes, batching.
- Hit-testing, click/hover/focus, keyboard.
- **Exit:** counter app works; only dirty nodes repaint (verified via debug overlay/log).

## M5 — Core primitives + A11y ✓
- `Pressable`, `Scroll`, `TextInput` (cursor, selection, IME), `List` (keyed).
- AccessKit integration.
- **Exit:** todo app (in-memory) fully keyboard + screen-reader usable on 1 desktop OS.

## M6 — Animation ✓
- Animated modifiers, springs, transitions, reduce-motion.
- **Exit:** list insert/remove animations, interruptible.

## M7 — Design system v0.1 ✓
- Tokens, light/dark, 10 components, gallery app, contrast tests.
- **Exit:** gallery shows all components/states; AA contrast tests pass.

## M8 — Data layer ✓
- `#[derive(Model)]`, `Handle<T>`, reactive `query`, SQLite backend, migrations v1.
- **Exit:** persistent todo app; UI updates on write; survives restart.

## M9 — Web ✓
- wasm shell, WebGPU + WebGL2 fallback, Canvas 2D text, in-memory data backend.
- **Exit:** same todo app compiles for wasm32; runs in browser via trunk.

## M10 — Mobile ✓
- Android, then iOS. Touch, gestures, safe areas, soft keyboard, lifecycle.
- **Exit:** todo app on device, both OSes.

## M11 — v0.1 public release ✓
- Docs site, 3 example apps, crates.io publish (`release-plz`).

## Later
- `view!{}` macro, `#[sihora::main]`, devtools, hot reload/preview, multi-window, more components, Elm/TCA crate.
