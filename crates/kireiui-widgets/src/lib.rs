//! Styled widget draw functions for Lumen.
//!
//! Each function consumes a [`Theme`] reference and produces
//! [`Vec<DrawCommand>`] using semantic tokens only — no raw colors.
//!
//! # Example
//! ```ignore
//! use kireiui_theme::Theme;
//! use kireiui_widgets::button;
//!
//! let theme = Theme::light();
//! let cmds = button::draw(
//!     &button::Props { label: "OK", variant: button::Variant::Primary, ..Default::default() },
//!     &theme,
//!     &mut text_ctx,
//! );
//! ```

pub mod button;
pub mod card;
pub mod checkbox;
pub mod dialog;
pub mod divider;
pub mod icon;
pub mod list_row;
pub mod text;
pub mod text_field;
pub mod toggle;

pub use kireiui_render::DrawCommand;
pub use kireiui_theme::Theme;
