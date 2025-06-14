use std::{env::current_dir, ops::Deref, sync::RwLock};

use maa_core::reload_core;
use maa_updater::{
    download_reporter::DefaultDownloadReporter,
    updater::{UpdateResult, Updater},
    version::{ClientVersionRequest, Versions},
};
use tauri::State;

use crate::{log_error_context, CommandResult};

pub const UPDATE_REPORT_EVENT: &str = "update-report";

pub struct VersionState(RwLock<Versions>);

impl Deref for VersionState {
    type Target = RwLock<Versions>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl VersionState {
    pub fn new(versions: Versions) -> Self {
        Self(RwLock::new(versions))
    }
}

#[tauri::command]
pub async fn update(
    target_type: ClientVersionRequest,
    updater: State<'_, Updater<DefaultDownloadReporter>>,
    versions: State<'_, VersionState>,
) -> CommandResult<UpdateResult> {
    let dst = current_dir().map_err(|e| log_error_context("获取CWD", e))?;
    let ver = versions.read().unwrap().client.clone();
    match updater.update(ver, target_type, &dst).await {
        Ok(res) => {
            if let UpdateResult::ClientSuccess(v) = &res {
                let mut guard = versions.write().unwrap();
                guard.client = v.clone();
                guard
                    .client
                    .write()
                    .map_err(|e| log_error_context("写入客户端配置", e))?;
                guard
                    .resource
                    .reload()
                    .map_err(|e| log_error_context("写入资源配置", e))?;
                reload_core().map_err(|e| log_error_context("重启MaaCore", e))?;
            }
            Ok(res)
        }
        Err(e) => {
            log_error_context("升级客户端", e);
            Err(())
        }
    }
}

#[tauri::command]
pub async fn update_resource(
    updater: State<'_, Updater<DefaultDownloadReporter>>,
    versions: State<'_, VersionState>,
) -> CommandResult<UpdateResult> {
    let dst = current_dir().map_err(|e| log_error_context("获取CWD", e))?;
    let ver = versions.read().unwrap().resource.clone();
    updater
        .update_resource(ver, &dst)
        .await
        .inspect(|res| {
            if let UpdateResult::ResourceSuccess(v) = res {
                versions.write().unwrap().resource = v.clone();
            }
        })
        .map_err(|e| log_error_context("升级资源", e))
}
