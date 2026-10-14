//! Platform shells: desktop (winit), web (wasm), android, iOS.

mod desktop;

pub use desktop::{run, run_app, run_reactive, run_with_text, AppFrame, TextContext};
