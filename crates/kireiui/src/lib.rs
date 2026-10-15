//! Lumen — declarative cross-platform UI framework in pure Rust.
//!
//! Facade crate that re-exports the public API from internal crates.

pub use kireidata as data;
pub use kireiui_core::a11y;
pub use kireiui_core::animation;
pub use kireiui_core::focus;
pub use kireiui_core::reactive;
pub use kireiui_core::text_input;
pub use kireiui_core::{
    Color, Event, Framed, Key, LayoutRect, Modifiers, NodeId, Padded, View, ViewExt, ViewTree,
};
pub use kireiui_layout::{Alignment, Constraints, EdgeInsets, HAlign, Point, Size, VAlign};
pub use kireiui_platform::{run, run_app, run_reactive, run_with_text, AppFrame, TextContext};
pub use kireiui_primitives::{hit_test, HStack, SizedBox, VStack, ZStack};
pub use kireiui_render::{DrawCommand, TextDraw};
#[cfg(not(target_arch = "wasm32"))]
pub use kireiui_text::TextBuffer;
pub use kireiui_theme::{self as theme, ColorScheme, Theme};
pub use kireiui_widgets as widgets;

#[cfg(target_os = "android")]
pub use kireiui_platform::{set_android_app, AndroidApp};
