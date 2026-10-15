# Architecture — Kirei

## 1. Layer overview
```
User app (components, ViewModels, Models)
──────────────────────────────────────────
kireiui           facade crate, re-exports, prelude
kireiui-widgets   styled components (read theme tokens)
kireiui-theme      tokens + default design system
kireiui-primitives headless: Rect, Text, Stack, Pressable, TextInput, Scroll
kireidata       Model store, Handle<T>, reactive queries, persistence
kireiui-core       View trait, view tree, reactive runtime, environment, events
kireiui-layout     layout engine
kireiui-text       shaping/layout/editing (cosmic-text or parley)
kireiui-render     display list → wgpu
kireiui-platform   shells: desktop (winit), web (wasm), android, ios
kireiui-macros     #[component], #[derive(Model)], #[kireiui::main], view!{}
```

## 2. Crate dependency rules
- Acyclic. `kireiui-core` depends on no other internal crate except `kireiui-layout` types (or none — keep layout behind trait).
- `kireiui-render` knows nothing about views; consumes a display list.
- `kireiui-platform` is the only crate touching winit / web-sys / JNI / UIKit.
- Users depend only on `kireiui`.
- `[workspace.dependencies]` pins shared versions.

## 3. Frame pipeline
```
Input event / signal write
   → mark dirty (fine-grained: only subscribed nodes)
   → rebuild dirty subtrees (component fns re-run only if their signals changed)
   → layout pass (dirty nodes + ancestors whose size changed)
   → paint pass → display list (damage regions only)
   → render: batch → GPU submit
   → a11y tree diff → AccessKit
```
- Driven by platform vsync / `request_redraw`.
- Idle = zero CPU (no continuous loop unless animating).

## 4. View model
- Retained tree. Nodes stored in arena (`SlotMap<NodeId, Node>`).
- `View` trait builds nodes; modifiers are wrapper views (generics for zero cost).
- Type erasure (`AnyView`) at component boundaries to control compile time.
- Keyed children for lists.

## 5. Reactive runtime
- Arena-owned signals; `Signal<T>` is a `Copy` ID handle.
- Dependency tracking via thread-local current observer.
- Owner scopes: signals/effects disposed when component unmounts.
- Batching: writes within an event handler flush once.

## 6. Layout
- Constraint-based (min/max size down, size up), Compose/SwiftUI style.
- Primitives: stack (H/V/Z), flex grow/shrink, grid, alignment, spacing, padding, frame, aspect ratio, safe area.
- Cache per node; invalidate on dirty.

## 7. Rendering
- wgpu backend. Instanced quads; SDF for rounded rects/borders/shadows.
- Glyph atlas (text), image atlas.
- Clip/transform stack; layers for opacity/animation.
- Damage tracking → scissor.

## 8. Text
- Shaping/layout via cosmic-text or parley (decide in ADR when reached).
- Editing: cursor, selection, undo, IME composition.
- Web: hidden `<textarea>` positioned over canvas for IME/clipboard.

## 9. Platform abstraction
```rust
trait Platform {
    fn run(app: impl App);
    fn request_redraw(&self, window: WindowId);
    fn surface(&self, window: WindowId) -> SurfaceTarget;
    fn clipboard(&self) -> &dyn Clipboard;
    fn ime(&self) -> &dyn Ime;
    fn spawn_local(&self, fut: LocalBoxFuture<'static, ()>);
}
```
- Desktop: winit. Web: wasm-bindgen + web-sys, canvas, hidden DOM. Mobile: winit android/ios backends or custom shells.

## 10. Web strategy
- Canvas + WebGPU (WebGL2 fallback).
- Hidden DOM overlay: text input, a11y (ARIA mirror), selection.
- Single-threaded by default; no blocking APIs in `core`.
- Size: `opt-level="z"`, LTO, `wasm-opt -Oz`.

## 11. Data layer
- `#[derive(Model)]` generates: schema, `Handle<T>` accessors (each field → signal), dirty tracking.
- In-memory cache = source of truth for UI (sync reads).
- Write-behind persistence queue (async).
- Backends behind trait: `sqlite` (native), `opfs`/`indexeddb` (web), `memory` (tests).
- Relations via `Handle<T>` IDs only.
- Migrations: `#[model(version = N)]` + migration fns.

## 12. Theme / design system
- `Theme` struct of semantic tokens, provided via environment.
- Widgets read only semantic tokens.
- Styles via traits (`ButtonStyle`, etc.) for per-widget override.
- Theme version pinned per major release.

## 13. Threading
- UI thread owns view tree + reactive runtime (`!Send`).
- Background work via executor; results posted back as signal writes.
- Renderer may move to separate thread later (desktop only).

## 14. Testing
- Unit: layout, reactivity, data.
- Headless render tests: render to texture → snapshot compare.
- Gallery app as manual/visual regression.
- CI: desktop build+test (3 OS), wasm32 compile check from day 1.
