#![deny(warnings)]

mod core;
mod updater;

use core::{get_config, run_daily, set_log_level, stop_core, update_config, ConfigState};
use std::env::set_current_dir;

use anyhow::Context;
use log4rs::{init_config, Handle};
use maa_cfg::Config;
use maa_core::tauri_logger::{log_config, LogHandleState};
use maa_updater::{updater::Updater, version::Versions};
use tauri::{utils::platform::current_exe, AppHandle, Manager};
use updater::{update, VersionState};

pub(crate) type CommandResult<T> = Result<T, String>;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> anyhow::Result<()> {
    init_cwd()?;
    // init states
    let config_state = Config::load(None).context("load configs")?;
    let version_state = Versions::load().context("load versions")?;
    // build app
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(ConfigState::new(config_state)) // TODO: 有没有必要不用mutex
        .manage(LogHandleState::default())
        .manage(Updater::default())
        .manage(VersionState::new(version_state))
        .setup(|app| {
            let log_handle = init_log(app.handle().clone())?;
            app.state::<LogHandleState>().get_or_init(|| log_handle);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            run_daily,
            stop_core,
            update_config,
            get_config,
            set_log_level,
            update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    Ok(())
}

fn init_log(handle: AppHandle) -> anyhow::Result<Handle> {
    let config = log_config(handle, log::LevelFilter::Info).context("get log config")?;
    init_config(config).context("init log with config")
}

fn init_cwd() -> anyhow::Result<()> {
    let exe = current_exe().context("get exe path")?;
    set_current_dir(exe.parent().unwrap()).context("set cwd")
}
