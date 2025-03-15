#![deny(warnings)]

mod core;
mod updater;

use core::{get_config, run_daily, set_log_level, stop_core, update_config};
use std::env::set_current_dir;

use anyhow::Context;
use log4rs::{init_config, Handle};
use maa_cfg::Config;
use maa_core::tauri_logger::{log_config, LogHandleState};
use maa_updater::{updater::Updater, version::Versions};
use tauri::{utils::platform::current_exe, AppHandle, Manager};
use updater::{update, update_resource, VersionState};

pub(crate) type CommandResult<T> = Result<T, ()>;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() -> anyhow::Result<()> {
    init_cwd()?;
    // init states
    let config_state = Config::load(None).await.context("load configs")?;
    let version_state = Versions::load().context("load versions")?;
    // build app
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(config_state)
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
            update_resource
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    Ok(())
}

fn init_log(handle: AppHandle) -> anyhow::Result<Handle> {
    let config = log_config(handle, log::LevelFilter::Trace).context("get log config")?;
    init_config(config).context("init log with config")
}

fn init_cwd() -> anyhow::Result<()> {
    let exe = current_exe().context("get exe path")?;
    set_current_dir(exe.parent().unwrap()).context("set cwd")
}

/// 使用debug!() 打印错误栈，并调用error!("{context}")。
/// 不返回错误
pub(crate) fn log_error_context<E: std::fmt::Debug>(context: &str, error: E) {
    log::error!("{context}");
    log::debug!("{error:?}");
}
