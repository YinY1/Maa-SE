use std::{env::current_dir, ops::Deref, sync::RwLock};

use anyhow::Context;
use log::info;
use maa_updater::{
    updater::{UpdateResult, Updater},
    version::{ClientVersionRequest, Versions},
};
use tauri::State;

use crate::CommandResult;

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
pub(crate) async fn update(
    target_type: ClientVersionRequest,
    updater: State<'_, Updater>,
    versions: State<'_, VersionState>,
) -> CommandResult<()> {
    let dst = current_dir().context("cwd").map_err(|e| format!("{e:?}"))?;
    let ver = versions.read().unwrap().client.clone();
    match updater.update(ver, target_type, &dst).await {
        Ok(res) => {
            info!("{}", res);
            if let UpdateResult::Success(v) = res {
                let mut guard = versions.write().unwrap();
                guard.client = v;
                guard.client.write().map_err(|e| format!("{e:?}"))?;
                guard.resource.reload().map_err(|e| format!("{e:?}"))?;
            }
            Ok(())
        }
        Err(e) => Err(format!("update error: {e:?}")),
    }
}

// TODO: resource
