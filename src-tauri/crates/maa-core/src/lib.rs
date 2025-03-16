#![deny(warnings)]
#![feature(once_cell_try)]

pub mod core;
#[cfg(feature = "tauri-handle")]
pub mod tauri_logger;

pub use core::*;

pub const ADB_PATH: &str = "D:\\MuMuPlayer-12.0\\shell\\adb.exe";
pub const DEFAULT_ADB_ADDRESS: &str = "127.0.0.1:16384";
