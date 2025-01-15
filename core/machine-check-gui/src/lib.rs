#![doc = include_str!("../README.md")]

/// The backend of the GUI, built on the same architecture as the rest of machine-check,
/// can interact with it normally.
#[cfg(not(target_arch = "wasm32"))]
mod backend;

/// The backend of the Graphical User Interface.
#[cfg(not(target_arch = "wasm32"))]
pub use backend::*;

/// The frontend of the GUI is built on the WebAssembly (WASM) architecture, running in a browser
/// and interacting with the backend.
mod frontend;

/// This include will emit an error if the frontend WASM is not properly built and prepared for use.
///
/// This is needed especially for development of machine-check-gui, where it is advantageous to let
/// build.rs compilation succeed even though there were errors building the frontend using WASM,
/// so that standard compilation can still proceed (useful especially when using rust-analyzer).
/// This include ensures that an error will still be emitted here.
///
/// In case you get an error here without an obvious error in the frontend, the postponed frontend
/// build errors still should be available as cargo build warnings if there are any.
///
/// See also build.rs of this package for the build implementation.
const _: &[u8] = include_bytes!("../content/wasm/machine_check_gui_wasm.js");
