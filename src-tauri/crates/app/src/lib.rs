#![deny(warnings)]

mod core;

use core::{run_daily, set_log_level, stop_core, update_config};
use std::sync::{Arc, Mutex, OnceLock};

use anyhow::Context;
use log4rs::{init_config, Handle};
use maa_cfg::Config;
use maa_core::tauri_logger::{log_config, LogHandleState};
use tauri::{AppHandle, Manager};

pub(crate) type CommandResult = Result<(), String>;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> anyhow::Result<()> {
    let config_state = Config::load(None).context("load configs")?;

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Arc::new(Mutex::new(config_state))) // TODO: 有没有必要不用mutex
        .manage(LogHandleState(OnceLock::new()))
        .setup(|app| {
            let log_handle = init_log(app.handle().clone())?;
            app.state::<LogHandleState>().0.get_or_init(|| log_handle);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            run_daily,
            stop_core,
            update_config,
            set_log_level,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    Ok(())
}

fn init_log(handle: AppHandle) -> anyhow::Result<Handle> {
    let config = log_config(handle, log::LevelFilter::Info).context("get log config")?;
    init_config(config).context("init log with config")
}
