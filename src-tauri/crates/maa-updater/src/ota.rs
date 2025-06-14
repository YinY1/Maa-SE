use std::path::Path;

use tempfile::tempdir;

use crate::{
    ZIP_FILE_SUFFIX, decompress,
    download_reporter::DownloadReporter,
    errors::{UpdateDetailedResult, UpdateErrorDetails},
    updater::{Details, Updater},
    version::ClientVersion,
};

pub const OTA_PREFIX: &str = "MAAComponent-OTA";

impl<R: DownloadReporter> Updater<R> {
    pub async fn download_ota_package(
        &self,
        current_version: &ClientVersion,
        details: &Details,
        dst: &Path,
    ) -> UpdateDetailedResult<()> {
        let temp_dir = tempdir().map_err(|e| UpdateErrorDetails::IOError {
            msg: "temp dir",
            source: e.into(),
        })?;
        let prefix = format!("{}-{}_", OTA_PREFIX, current_version.version().unwrap());
        let file = self
            .download_package(&prefix, ZIP_FILE_SUFFIX, details, temp_dir)
            .await?;
        decompress(file, dst.to_path_buf()).await
    }

    // TODO: 使用git增量更新资源？
    pub async fn download_ota_resource() -> UpdateDetailedResult<()> {
        unimplemented!()
    }
}
