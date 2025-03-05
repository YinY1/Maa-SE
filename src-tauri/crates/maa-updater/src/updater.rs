use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

use anyhow::Context;
use fs_extra::dir::{CopyOptions, move_dir};
use log::{debug, trace, warn};
use reqwest::header::ACCEPT;
use semver::Version;
use serde::{Deserialize, de::DeserializeOwned};
use strum::Display;
use tempfile::{TempDir, tempdir};
use tokio::{fs::File, io::AsyncWriteExt, join, task::spawn_blocking};

use crate::{
    GITHUB_RESOURCE_URL, RESOURCE_SUMMARY, VERSION_SUMMARY, ZIP_FILE_SUFFIX, decompress,
    version::{ClientVersion, ClientVersionRequest, ResourceVersion},
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
    ClientSuccess(ClientVersion),
    ResourceSuccess(ResourceVersion), // TODO: box代替避免过大
}

pub struct UpdaterGuard<'a>(&'a AtomicBool);

impl Drop for UpdaterGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
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

    pub fn lock(&self) -> Result<UpdaterGuard<'_>, bool> {
        self.updating
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| UpdaterGuard(&self.updating))
    }

    pub async fn get_object<T: DeserializeOwned>(&self, url: &str) -> anyhow::Result<T> {
        self.client
            .get(url)
            .send()
            .await?
            .error_for_status()
            .context("get response")?
            .json()
            .await
            .context("serde json")
    }

    /// download update files to `dst` if `current_version` is not latest
    pub async fn update(
        &self,
        current_version: ClientVersion,
        target_type: ClientVersionRequest,
        dst: &Path,
    ) -> anyhow::Result<UpdateResult> {
        let _guard = match self.lock() {
            Ok(g) => g,
            Err(_) => return Ok(UpdateResult::Updating),
        };
        self.update_impl(current_version, target_type, dst).await
    }

    pub async fn update_resource(
        &self,
        current_version: ResourceVersion,
        dst: &Path,
    ) -> anyhow::Result<UpdateResult> {
        let _guard = match self.lock() {
            Ok(g) => g,
            Err(_) => return Ok(UpdateResult::Updating),
        };
        self.update_resource_impl(current_version, dst).await
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
                    return Ok(UpdateResult::ClientSuccess(
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
            Ok(_) => Ok(UpdateResult::ClientSuccess(
                target_type.to_version(details.version),
            )),
            Err(e) => {
                warn!("full-update failed: {}", e.root_cause());
                debug!("full-update trace: {e:?}");
                Err(e)
            }
        }
    }

    async fn update_resource_impl(
        &self,
        current_version: ResourceVersion,
        dst: &Path,
    ) -> anyhow::Result<UpdateResult> {
        let version = match self
            .check_resource_update(&current_version)
            .await
            .context("check resource update")?
        {
            Some(v) => v,
            None => return Ok(UpdateResult::AlreadyUpdated),
        };

        match self
            .download_full_resource(dst)
            .await
            .context("download resource")
        {
            Ok(_) => Ok(UpdateResult::ResourceSuccess(version)),
            Err(e) => {
                warn!("resource update failed: {}", e.root_cause());
                debug!("resource update trace: {e:?}");
                Err(e)
            }
        }
    }

    /// check from maa api, return Ok(Some(Details)) if needs updating
    pub async fn check_core_update_and_get_details(
        &self,
        current_version: &ClientVersion,
        target_type: &ClientVersionRequest,
    ) -> anyhow::Result<Option<Details>> {
        trace!("get version summary from maa api");
        let info: VersionInfo = self
            .get_object(VERSION_SUMMARY)
            .await
            .context("get version info")?;

        let summary = match target_type {
            ClientVersionRequest::Nightly => info.alpha,
            ClientVersionRequest::Beta => info.beta,
            ClientVersionRequest::Stable => info.stable,
        };

        let details_url = match current_version {
            ClientVersion::Unknown => summary.detail_url,
            _ => {
                let cur_ver = current_version.semver().context("parse current version")?;
                let target_ver = Version::parse(summary.version.trim_start_matches('v'))
                    .context("parse target version")?;
                if cur_ver >= target_ver {
                    return Ok(None);
                }
                summary.detail_url
            }
        };

        trace!("get version details from maa api");
        let details = self.get_object(&details_url).await.context("get details")?;

        Ok(Some(details))
    }

    /// check from maa api, return Ok(Some(resource info)) if needs updating
    pub async fn check_resource_update(
        &self,
        current_version: &ResourceVersion,
    ) -> anyhow::Result<Option<ResourceVersion>> {
        trace!("get resource version from maa api");
        let latest_version: ResourceVersion = self
            .get_object(RESOURCE_SUMMARY)
            .await
            .context("get summary")?;

        if current_version.last_updated == latest_version.last_updated {
            return Ok(None);
        }
        let current = current_version
            .timestamp()
            .context("parse current timestamp")?;
        let latest = latest_version
            .timestamp()
            .context("parse latest timestamp")?;

        Ok((current < latest).then_some(latest_version))
    }
}

/// download
impl Updater {
    pub async fn download_full_package(&self, details: &Details, dst: &Path) -> anyhow::Result<()> {
        // TODO: 使用tempdir in 避免意外关闭时没删除临时目录，可以后期手动删除
        let temp_dir = tempdir().context("create temp dir")?;
        let file = self
            .download_package(MAA_PKG_PREFIX, ZIP_FILE_SUFFIX, details, temp_dir)
            .await
            .context("download zip")?;

        decompress(file, dst.to_path_buf())
            .await
            .context("decompress")
    }

    pub async fn download_full_resource(&self, dst: &Path) -> anyhow::Result<()> {
        let temp_dir = tempdir().context("create temp dir")?;
        let temp_path = temp_dir.path();
        let path = temp_path.join("resources");
        let file = self
            .download_chunks(GITHUB_RESOURCE_URL, &path)
            .await
            .context("download zip")?;

        decompress(file, temp_path.to_path_buf())
            .await
            .context("decompress")?;

        let resources_path = temp_path.join(RESOURCE_REPO_NAME);
        let (s1, s2) = join!(
            move_dir_async(resources_path.join("cache"), dst.to_path_buf()),
            move_dir_async(resources_path.join("resource"), dst.to_path_buf())
        );
        s1.context("movwe cache")?;
        s2.context("move resource")?;
        Ok(())
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
            // TODO: 添加下载进度回报
            file.write_all(&chunk).await?;
        }
        trace!("download finish");
        file.flush().await?;

        Ok(file.into_std().await)
    }
}

async fn move_dir_async(from: PathBuf, to: PathBuf) -> anyhow::Result<u64> {
    trace!("move dir from `{from:?}` to `{to:?}`");
    spawn_blocking(move || {
        let options = CopyOptions::new().overwrite(true);
        move_dir(from, to, &options)
    })
    .await
    .context("spawn blocking handle")?
    .context("move dir")
}
