//! Lumen — declarative cross-platform UI framework in pure Rust.
//!
//! Facade crate that re-exports the public API from internal crates.

pub use lumen_core::Color;
pub use lumen_platform::run;
pub use lumen_render::DrawCommand;
