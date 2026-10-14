//! Lumen — declarative cross-platform UI framework in pure Rust.
//!
//! Facade crate that re-exports the public API from internal crates.

pub use lumen_core::reactive;
pub use lumen_core::{
    Color, Event, Framed, Key, LayoutRect, NodeId, Padded, View, ViewExt, ViewTree,
};
pub use lumen_layout::{Alignment, Constraints, EdgeInsets, HAlign, Point, Size, VAlign};
pub use lumen_platform::{run, run_reactive, run_with_text, TextContext};
pub use lumen_primitives::{HStack, SizedBox, VStack, ZStack};
pub use lumen_render::DrawCommand;
pub use lumen_text::TextBuffer;
