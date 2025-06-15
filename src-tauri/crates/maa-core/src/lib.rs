#![deny(warnings)]
#![feature(once_cell_try)]

pub mod core;
#[cfg(feature = "tauri-handle")]
pub mod tauri_logger;

pub use core::*;
