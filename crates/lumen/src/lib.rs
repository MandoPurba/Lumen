//! Lumen — declarative cross-platform UI framework in pure Rust.
//!
//! Facade crate that re-exports the public API from internal crates.

pub use lumen_core::{Color, Framed, LayoutRect, NodeId, Padded, View, ViewExt, ViewTree};
pub use lumen_layout::{Alignment, Constraints, EdgeInsets, HAlign, Point, Size, VAlign};
pub use lumen_platform::run;
pub use lumen_primitives::{HStack, SizedBox, VStack, ZStack};
pub use lumen_render::DrawCommand;
