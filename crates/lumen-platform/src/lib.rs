//! Platform shells: desktop (winit), web (wasm), android, iOS.

mod common;

#[cfg(not(target_arch = "wasm32"))]
mod desktop;
#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(not(target_arch = "wasm32"))]
pub use desktop::{run, run_app, run_reactive, run_with_text, AppFrame, TextContext};
#[cfg(target_arch = "wasm32")]
pub use web::{run, run_app, run_reactive, run_with_text, AppFrame, TextContext};
