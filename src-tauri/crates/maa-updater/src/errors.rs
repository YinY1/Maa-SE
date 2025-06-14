use thiserror::Error;
use tokio::task::JoinError;
use zip::result::ZipError;

#[derive(Debug, Error)]
pub enum UpdateError {
    #[error("full update error: {0}")]
    FullUpdateError(UpdateErrorType),
    #[error("full update error: {0}")]
    OTAError(UpdateErrorType),
}

#[derive(Debug, Error)]
pub enum UpdateErrorType {
    #[error("package update error: {0}")]
    PackageUpdateError(UpdateErrorDetails),

    #[error("resource update error: {0}")]
    ResourceUpdateError(UpdateErrorDetails),
}

pub type UpdateDetailedResult<T> = Result<T, UpdateErrorDetails>;

#[derive(Debug, Error)]
pub enum UpdateErrorDetails {
    // TODO: linux targz error
    #[cfg(not(target_os = "linux"))]
    #[error("decompress error: {0}")]
    DecompressError(#[from] ZipError),

    #[error("IO error: {msg}")]
    IOError {
        msg: &'static str,
        #[backtrace]
        source: fs_extra::error::Error,
    },

    #[error("version error: {0}")]
    VersionError(&'static str),

    #[error("download error: {0}")]
    DownloadError(#[from] reqwest::Error),

    #[error("tokio error: {0}")]
    TokioError(#[from] JoinError),
}
