# Lumen

Declarative, cross-platform UI framework in pure Rust.

SwiftUI/Compose-style API. Own GPU renderer (wgpu). Own design system. Built-in reactive data layer. No HTML/CSS.

One codebase &rarr; desktop, web, mobile. Pixel-consistent.

## Status

**M0 — Bootstrap.** Workspace scaffolded, CI passing on empty crates.

## Building

```sh
cargo build --workspace
cargo test --workspace
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
```

### Wasm check

```sh
cargo check -p lumen-core --target wasm32-unknown-unknown
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
