#![deny(warnings)]
#![feature(error_generic_member_access)]

pub mod errors;
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
pub mod ota;
pub mod updater;
pub mod version;

use std::{fs::File, path::PathBuf};

use errors::{UpdateDetailedResult, UpdateErrorDetails};
use tokio::task::spawn_blocking;

pub const VERSION_SUMMARY: &str =
    "https://ota.maa.plus/MaaAssistantArknights/api/version/summary.json";
pub const RESOURCE_SUMMARY: &str =
    "https://ota.maa.plus/MaaAssistantArknights/MaaAssistantArknights/resource/version.json";
#[cfg(not(target_os = "linux"))]
pub const GITHUB_RESOURCE_URL: &str =
    "https://github.com/MaaAssistantArknights/MaaResource/archive/refs/heads/main.zip";

#[cfg(target_os = "linux")]
pub const GITHUB_RESOURCE_URL: &str =
    "https://github.com/MaaAssistantArknights/MaaResource/archive/refs/heads/main.tar.gz";

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
pub const ZIP_FILE_SUFFIX: &str = "win-x64.zip";

#[cfg(all(target_os = "windows", target_arch = "arm"))]
pub const ZIP_FILE_SUFFIX: &str = "win-arm64.zip";

#[cfg(target_os = "linux")]
pub const ZIP_FILE_SUFFIX: &str = constcat::concat!("linux-", std::env::consts::ARCH, ".tar.gz");

#[cfg(target_os = "macos")]
pub const ZIP_FILE_SUFFIX: &str = "macos-runtime-universal.zip";

pub(crate) async fn decompress(file: File, dst: PathBuf) -> UpdateDetailedResult<()> {
    log::trace!("decompress file `{:?}` to dst: `{:?}`", file, dst);
    match spawn_blocking(move || decompress_impl(file, dst)).await {
        Ok(res) => res,
        Err(e) => Err(UpdateErrorDetails::TokioError(e)),
    }
}

#[cfg(not(target_os = "linux"))]
fn decompress_impl(file: File, dst: PathBuf) -> UpdateDetailedResult<()> {
    let mut archive = zip::ZipArchive::new(file).map_err(UpdateErrorDetails::DecompressError)?;
    archive
        .extract(dst)
        .map_err(UpdateErrorDetails::DecompressError)
}

// TODO: Error Type
#[cfg(target_os = "linux")]
fn decompress_impl(file: File, dst: PathBuf) -> UpdateDetailedResult<()> {
    let gz = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(gz);
    archive.unpack(dst).context("unpack tar.gz")
}
