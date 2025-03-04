use std::{ops::Deref, sync::Mutex};

use anyhow::Context;
use maa_cfg::{Config, Parameters};
use maa_core::tauri_logger::{log_config, LogHandleState};
use tauri::{async_runtime::spawn_blocking, AppHandle, State};

use crate::CommandResult;

pub struct ConfigState(Mutex<Config>);

impl Deref for ConfigState {
    type Target = Mutex<Config>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ConfigState {
    pub fn new(config: Config) -> Self {
        Self(Mutex::new(config))
    }
}

#[tauri::command]
pub(crate) async fn run_daily(configs: State<'_, ConfigState>) -> CommandResult<()> {
    let tasks = configs.lock().unwrap().available_daily_tasks();
    // TODO: 验证blocking的合理性
    spawn_blocking(move || maa_core::run_core_tauri(tasks))
        .await
        .unwrap()
        .map_err(|e| format!("{e:?}"))
}

#[tauri::command]
pub(crate) fn stop_core() {
    maa_core::stop_core();
}

/// update config json with given  name and params.
///
/// see `maa_cfg::ConfigType`.
#[tauri::command]
pub(crate) fn update_config(
    name: String,
    params: Parameters,
    configs: State<'_, ConfigState>,
) -> CommandResult<()> {
    let cfg_type = name.parse().unwrap();
    configs
        .lock()
        .unwrap()
        .set_and_write(cfg_type, params)
        .context("update config")
        .map_err(|e| format!("{e:?}"))
}

#[tauri::command]
pub(crate) fn get_config(configs: State<'_, ConfigState>) -> CommandResult<String> {
    serde_json::to_string(configs.inner().lock().unwrap().deref())
        .context("serialize configs to string")
        .map_err(|e| format!("{e:?}"))
}

#[tauri::command]
pub(crate) fn set_log_level(
    level: &str,
    app_handle: AppHandle,
    log_handle: State<'_, LogHandleState>,
) -> CommandResult<()> {
    let level = level
        .parse()
        .map_err(|e: log::ParseLevelError| e.to_string())?;
    let config = log_config(app_handle, level).map_err(|e| format!("{e:?}"))?;
    log_handle.get().unwrap().set_config(config);
    Ok(())
}
