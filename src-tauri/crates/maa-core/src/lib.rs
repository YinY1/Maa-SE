#![deny(warnings)]

use std::sync::{Arc, Mutex};

use maa_cfg::Config;

pub mod callback;
pub mod core;
#[cfg(feature = "tauri-handle")]
pub mod tauri_logger;

pub use core::*;

pub const ADB_PATH: &str = "D:\\MuMuPlayer-12.0\\shell\\adb.exe";
pub const DEFAULT_ADB_ADDRESS: &str = "127.0.0.1:16384";

pub type ConfigState = Arc<Mutex<Config>>;
