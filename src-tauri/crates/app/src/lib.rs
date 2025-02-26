#![deny(warnings)]

mod core;

use core::{run_daily, stop_core, update_config};
use std::sync::{Arc, Mutex};

use anyhow::Context;
use maa_cfg::Config;
use maa_core::tauri_logger::Logger;
use tauri::AppHandle;

pub(crate) type CommandResult = Result<(), String>;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> anyhow::Result<()> {
    let config_state = Config::load(None).context("load configs")?;
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            build_logger(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            run_daily,
            stop_core,
            update_config
        ])
        .manage(Arc::new(Mutex::new(config_state)))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    Ok(())
}

fn build_logger(handle: AppHandle) {
    let logger = Logger::new(handle);
    env_logger::Builder::from_default_env()
        .target(env_logger::Target::Pipe(Box::new(logger)))
        .init();
}
