//! Lumen — declarative cross-platform UI framework in pure Rust.
//!
//! Facade crate that re-exports the public API from internal crates.

pub use lumen_core::a11y;
pub use lumen_core::focus;
pub use lumen_core::reactive;
pub use lumen_core::text_input;
pub use lumen_core::{
    Color, Event, Framed, Key, LayoutRect, Modifiers, NodeId, Padded, View, ViewExt, ViewTree,
};
pub use lumen_layout::{Alignment, Constraints, EdgeInsets, HAlign, Point, Size, VAlign};
pub use lumen_platform::{run, run_app, run_reactive, run_with_text, AppFrame, TextContext};
pub use lumen_primitives::{hit_test, HStack, SizedBox, VStack, ZStack};
pub use lumen_render::DrawCommand;
pub use lumen_text::TextBuffer;
