use std::path::Path;

use anyhow::Context;
use tempfile::tempdir;

use crate::{
    ZIP_FILE_SUFFIX, decompress,
    updater::{Details, Updater},
    version::ClientVersion,
};

pub const OTA_PREFIX: &str = "MAAComponent-OTA";

impl Updater {
    pub async fn download_ota_package(
        &self,
        current_version: &ClientVersion,
        details: &Details,
        dst: &Path,
    ) -> anyhow::Result<()> {
        let temp_dir = tempdir().context("create temp dir")?;
        let prefix = format!("{}-{}_", OTA_PREFIX, current_version.version().unwrap());
        let file = self
            .download_package(&prefix, ZIP_FILE_SUFFIX, details, temp_dir)
            .await
            .context("download zip")?;
        decompress(file, dst).context("decompress")
    }

    // TODO: 使用git增量更新资源？
    pub async fn download_ota_resource() {
        unimplemented!()
    }
}
