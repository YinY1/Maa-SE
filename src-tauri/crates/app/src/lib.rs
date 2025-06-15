#![deny(warnings)]

mod core;
mod updater;

use core::{get_config, run_daily, set_log_level, stop_core, update_config};
use std::{env::set_current_dir, sync::Arc, time::Duration};

use anyhow::Context;
use log::error;
use log4rs::{init_config, Handle};
use maa_cfg::Config;
use maa_core::tauri_logger::log_config;
use maa_updater::{
    download_reporter::DefaultDownloadReporter, updater::Updater, version::Versions,
};
use tauri::{utils::platform::current_exe, AppHandle, Emitter, Manager};
use updater::{update, update_resource, VersionState};

use crate::updater::UPDATE_REPORT_EVENT;

const DOWNLOAD_REPORT_INTERVAL: Duration = Duration::from_millis(500);

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
        .manage(Arc::new(config_state))
        .manage(VersionState::new(version_state))
        .setup(|app| {
            app.manage(init_log(app.handle().clone())?);
            app.manage(init_updater(app.handle().clone()));
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

fn init_updater(handle: AppHandle) -> Updater<DefaultDownloadReporter> {
    Updater::new(DefaultDownloadReporter::new(
        DOWNLOAD_REPORT_INTERVAL,
        Some(move |downloaded, total| {
            let h = handle.clone();
            async move {
                if let Err(e) = h.emit(UPDATE_REPORT_EVENT, (downloaded, total)) {
                    error!("Failed to emit update report: {}", e);
                }
            }
        }),
    ))
}

/// 使用debug!() 打印错误栈，并调用error!("{context}")。
/// 不返回错误
pub(crate) fn log_error_context<E: std::fmt::Debug>(context: &str, error: E) {
    log::error!("{context}");
    log::debug!("{error:?}");
}
