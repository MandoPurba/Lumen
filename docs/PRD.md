# PRD — Sihora Framework

> Working name: **Sihora**. Rename via find/replace.
> Status: Draft v0.1 · Owner: you · Last updated: 2026-10-14

## 1. Vision
A declarative, cross-platform UI framework in pure Rust. SwiftUI/Compose-style API. No HTML/CSS. Own GPU renderer, own design system, built-in reactive data layer (SwiftData-like).

One codebase → desktop, web, mobile. Pixel-consistent.

## 2. Problem
- Rust UI ecosystem is fragmented; no option combines: declarative modifier API + fine-grained reactivity + own renderer + built-in persistence + first-class design system.
- Webview-based stacks (Tauri/Electron) are heavy and not native-feeling.
- Flutter/Compose require Dart/Kotlin; Rust devs want one language end-to-end.

## 3. Target users
- **Primary:** Rust developers building app-like UIs (tools, dashboards, editors, internal apps).
- **Secondary:** Teams wanting one codebase across desktop + web + mobile.
- **Not targeted:** Content/SEO websites (use DOM frameworks).

## 4. Goals
- G1: SwiftUI/Compose-level DX in Rust (declarative, modifiers, signals).
- G2: Single renderer, identical output on all platforms.
- G3: Built-in data layer: `#[derive(Model)]`, reactive queries, persistence.
- G4: Own design system as default; fully replaceable or removable.
- G5: Small binaries, fast startup, 60fps+ (120fps capable).

## 5. Non-goals (v1)
- Native OS widgets.
- HTML/CSS/DOM rendering model.
- SEO / server-side rendering.
- Visual designer / WYSIWYG tool.
- Forcing an app architecture (MVVM is default, not required).

## 6. Functional requirements

### 6.1 App model
- `App` trait + `fn main() { sihora::run(MyApp) }`.
- Later: `#[sihora::main]` macro generating desktop/wasm/mobile entry glue.
- Multi-window (desktop).

### 6.2 View system
- `View` trait, composable components (functions returning `impl View`).
- Modifiers: `.padding()`, `.background()`, `.frame()`, `.corner_radius()`, `.on_tap()`, etc.
- Containers: `VStack`, `HStack`, `ZStack`, `Grid`, `List`, `Scroll`.
- Conditionals and lists with stable keys.

### 6.3 Reactivity
- `Signal<T>`, `Memo<T>`, `Effect`. Copy handles (no `clone()` spam in closures).
- Fine-grained updates: only dirty nodes relayout/repaint.
- Environment/context propagation (theme, locale, store).

### 6.4 Layout
- Own engine: stack/flex/grid subset, intrinsic sizing, alignment, spacing, safe areas.

### 6.5 Rendering
- GPU via wgpu. Rects, rounded rects (SDF), borders, shadows, gradients, images, clipping, text.
- Damage-region repaint.

### 6.6 Text
- Shaping, wrapping, bidi, emoji, font fallback, selection, editing, IME.

### 6.7 Input & events
- Mouse, touch, keyboard, scroll, focus/tab order, gestures (tap, drag, long press), shortcuts.

### 6.8 Accessibility
- Accessibility tree via AccessKit. Screen reader support all platforms.

### 6.9 Animation
- `.animate(spring())` / transitions on modifier values. Interruptible.

### 6.10 Data layer (SwiftData-like)
- `#[derive(Model)]` → schema, accessors, change tracking.
- `Handle<T>` (Copy, ID-based), no `Rc<RefCell<T>>`.
- `query::<T>()` returns reactive result; UI auto-updates.
- Backends: SQLite (native), OPFS/IndexedDB (web).
- Versioned migrations.

### 6.11 Design system
- Token-based theme (color, type, spacing, radius, elevation, motion), light + dark.
- Styled components on top of headless primitives.
- Opt-out: `default-features = false` → primitives only.
- Theme versioning; upgrades opt-in.

### 6.12 Platforms
- Desktop: macOS, Windows, Linux.
- Web: wasm32 + WebGPU (WebGL2 fallback), canvas + hidden DOM for input/a11y.
- Mobile: Android, iOS.

### 6.13 Tooling
- Component gallery app.
- Devtools (view tree, layout bounds, repaint flashing) — later.
- Hot reload / preview — later.

## 7. Non-functional requirements
| Area | Target |
|---|---|
| Frame time | < 8ms typical (120fps capable) |
| Cold start (desktop) | < 200ms |
| Hello-world binary (desktop, release) | < 8MB |
| Wasm bundle (hello world, gz) | < 1.5MB |
| Memory (hello world) | < 50MB |
| A11y | WCAG 2.2 AA contrast for all default tokens |
| MSRV | Latest stable − 2 |

## 8. Success metrics
- M1 demo: counter app desktop, 60fps, < 500 LOC user code for todo app.
- External: 3 non-trivial example apps; first outside contributor; crates.io release.

## 9. Milestones
See `ROADMAP.md`.

## 10. Risks
| Risk | Impact | Mitigation |
|---|---|---|
| Scope too large (solo) | Project stalls | Strict milestones, desktop first, ≤10 components v0.1 |
| Text input / IME | Unusable for non-Latin | Use cosmic-text/parley; hidden DOM on web; test CJK early |
| Rust closure/ownership DX | Adoption | Copy signal handles; arena-based reactive runtime |
| Compile times (generic view types) | DX | Type-erase at component boundaries |
| Accessibility | Unshippable for real apps | AccessKit from M5, not later |
| Mobile lifecycle | Late surprises | Keep platform crate isolated; wasm CI from day 1 |
| Design quality | Looks amateur | Token discipline + gallery + visual tests |

## 11. Open questions
- Final name.
- Builder API only, or `view!{}` macro in v1?
- License: MIT/Apache-2.0 dual (default Rust convention)?
- Default font (Inter / Geist, both OFL).
