use std::{
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

use anyhow::Context;
use log::{debug, trace, warn};
use reqwest::header::ACCEPT;
use semver::Version;
use serde::Deserialize;
use strum::Display;
use tempfile::{TempDir, tempdir};
use tokio::{
    fs::{File, rename},
    io::AsyncWriteExt,
};

use crate::{
    GITHUB_RESOURCE_URL, VERSION_SUMMARY, ZIP_FILE_SUFFIX, decompress,
    version::{ClientVersion, ClientVersionRequest},
};

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36 Edg/133.0.0.0";
const HEADER_DOWNLOAD: &str = "application/octet-stream";

const MAA_PKG_PREFIX: &str = "MAA-";
const RESOURCE_REPO_NAME: &str = "MaaResource-main";

#[derive(Deserialize)]
pub struct Summary {
    version: String,
    #[serde(rename = "detail")]
    detail_url: String,
}

#[derive(Deserialize)]
pub struct VersionInfo {
    alpha: Summary,
    beta: Summary,
    stable: Summary,
}

#[derive(Deserialize)]
pub struct Details {
    pub version: String,
    #[serde(rename = "details")]
    pub inner: DetailsInner,
}

#[derive(Deserialize)]
pub struct DetailsInner {
    pub tag_name: String,
    pub assets: Vec<Asset>,
}

#[derive(Deserialize)]
pub struct Asset {
    pub name: String,
    pub size: usize,
    #[serde(rename = "browser_download_url")]
    pub download_url: String,
}

#[derive(Display)]
pub enum UpdateResult {
    Updating,
    AlreadyUpdated,
    Success(ClientVersion),
}

pub struct Updater {
    client: reqwest::Client,
    updating: AtomicBool,
}

impl Default for Updater {
    fn default() -> Self {
        Self::new()
    }
}

impl Updater {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .build()
                .unwrap(),
            updating: AtomicBool::new(false),
        }
    }

    /// download update files to `dst` if `current_version` is not latest
    pub async fn update(
        &self,
        current_version: ClientVersion,
        target_type: ClientVersionRequest,
        dst: &Path,
    ) -> anyhow::Result<UpdateResult> {
        if self
            .updating
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Ok(UpdateResult::Updating);
        }
        let res = self.update_impl(current_version, target_type, dst).await;
        self.updating.store(false, Ordering::Release);
        res
    }

    async fn update_impl(
        &self,
        current_version: ClientVersion,
        target_type: ClientVersionRequest,
        dst: &Path,
    ) -> anyhow::Result<UpdateResult> {
        let details = match self
            .check_core_update_and_get_details(&current_version, &target_type)
            .await
            .context("check update")?
        {
            Some(d) => d,
            None => return Ok(UpdateResult::AlreadyUpdated),
        };

        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        if !matches!(current_version, ClientVersion::Unknown) {
            debug!("start download ota");
            match self
                .download_ota_package(&current_version, &details, dst)
                .await
                .context("download ota")
            {
                Ok(_) => {
                    return Ok(UpdateResult::Success(
                        target_type.to_version(details.version),
                    ));
                }
                Err(e) => {
                    warn!("ota failed: {}", e.root_cause());
                    debug!("ota failed trace: {e:?}");
                }
            }
        }

        debug!("start download full package");
        match self
            .download_full_package(&details, dst)
            .await
            .context("download full pkg")
        {
            Ok(_) => Ok(UpdateResult::Success(
                target_type.to_version(details.version),
            )),
            Err(e) => {
                warn!("full-update failed: {}", e.root_cause());
                debug!("full-update trace: {e:?}");
                Err(e)
            }
        }
    }

    /// check from maa api, return Ok(Some(Details)) if needs update
    pub async fn check_core_update_and_get_details(
        &self,
        current_version: &ClientVersion,
        target_type: &ClientVersionRequest,
    ) -> anyhow::Result<Option<Details>> {
        trace!("get version summary from maa api");
        let info: VersionInfo = self
            .client
            .get(VERSION_SUMMARY)
            .send()
            .await?
            .error_for_status()
            .context("get summary.json")?
            .json()
            .await
            .context("serde json")?;

        let summary = match target_type {
            ClientVersionRequest::Nightly => info.alpha,
            ClientVersionRequest::Beta => info.beta,
            ClientVersionRequest::Stable => info.stable,
        };

        let target_ver = Version::parse(summary.version.trim_start_matches('v'))
            .context("parse target version")?;
        let details_url = match current_version {
            ClientVersion::Nightly(v) | ClientVersion::Beta(v) | ClientVersion::Stable(v) => {
                let cur_ver =
                    Version::parse(v.trim_start_matches('v')).context("parse current version")?;
                if cur_ver >= target_ver {
                    return Ok(None);
                }
                summary.detail_url
            }
            ClientVersion::Unknown => summary.detail_url,
        };

        trace!("get version details from maa api");
        let details = self
            .client
            .get(details_url)
            .send()
            .await?
            .error_for_status()
            .context("get details.json")?
            .json()
            .await
            .context("serde details.json")?;

        Ok(Some(details))
    }

    pub async fn download_full_package(&self, details: &Details, dst: &Path) -> anyhow::Result<()> {
        // TODO: 使用tempdir in 避免意外关闭时没删除临时目录，可以后期手动删除
        let temp_dir = tempdir().context("create temp dir")?;
        let file = self
            .download_package(MAA_PKG_PREFIX, ZIP_FILE_SUFFIX, details, temp_dir)
            .await
            .context("download zip")?;

        decompress(file, dst).context("decompress")
    }

    pub async fn download_full_resource(&self, dst: &Path) -> anyhow::Result<()> {
        let temp_dir = tempdir().context("create temp dir")?;
        let temp_path = temp_dir.path();
        let path = temp_path.join("resources");
        let file = self
            .download_chunks(GITHUB_RESOURCE_URL, &path)
            .await
            .context("download zip")?;

        decompress(file, temp_path).context("decompress")?;
        let resources_path = temp_path.join(RESOURCE_REPO_NAME);
        rename(resources_path.join("cache"), dst)
            .await
            .context("move cache")?;
        rename(resources_path.join("resource"), dst)
            .await
            .context("move resource")
    }

    /// download package with given format,
    /// return the archive file and it's version
    pub async fn download_package(
        &self,
        prefix: &str,
        suffix: &str,
        details: &Details,
        temp_dir: TempDir,
    ) -> anyhow::Result<std::fs::File> {
        let name = format!("{}{}-{}", prefix, details.version, suffix);
        trace!("try to find url of asset `{name}`");
        let url = details
            .inner
            .assets
            .iter()
            .find_map(|asset| (asset.name == name).then_some(&asset.download_url))
            .ok_or(anyhow::anyhow!("no match package"))?;

        let temp_path = temp_dir.path().join(name);
        self.download_chunks(url, &temp_path).await
    }

    pub async fn download_chunks(&self, url: &str, dst: &Path) -> anyhow::Result<std::fs::File> {
        trace!("start download to `{dst:?}` from `{url}`");
        let mut resp = self
            .client
            .get(url)
            .header(ACCEPT, HEADER_DOWNLOAD)
            .send()
            .await?
            .error_for_status()
            .context("fetch release")?;

        let mut file = File::options()
            .create_new(true)
            .read(true)
            .append(true)
            .open(dst)
            .await
            .context("create target file")?;
        while let Some(chunk) = resp.chunk().await? {
            file.write_all(&chunk).await?;
        }
        trace!("download finish");
        file.flush().await?;

        Ok(file.into_std().await)
    }
}
