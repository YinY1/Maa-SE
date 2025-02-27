#[cfg(feature = "dynamic-log-level")]
pub use dynamic_log::*;
use tauri::AppHandle;

const CALLBACK_EVENT: &str = "callback-log";

#[derive(Debug)]
pub struct Logger {
    app: AppHandle,
}

impl Logger {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

#[cfg(not(feature = "dynamic-log-level"))]
impl std::io::Write for Logger {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let content = str::from_utf8(buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        match tauri::Emitter::emit(&self.app, CALLBACK_EVENT, content) {
            Ok(_) => Ok(buf.len()),
            Err(e) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            )),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(feature = "dynamic-log-level")]
mod dynamic_log {
    use std::sync::OnceLock;

    use anyhow::Context;
    use log::LevelFilter;
    use log4rs::{
        Config, Handle,
        append::{Append, file::FileAppender},
        config::{Appender, Root},
    };
    use tauri::{AppHandle, Emitter};

    use super::{CALLBACK_EVENT, Logger};

    const FILE_APPENDER_NAME: &str = "file";
    const CALLBACK_APPENDER_NAME: &str = "callback";
    const LOG_FILE_NAME: &str = "maa-se.log";

    pub struct LogHandleState(pub OnceLock<Handle>);

    impl Append for Logger {
        fn append(&self, record: &log::Record) -> anyhow::Result<()> {
            let content = match record.args().as_str() {
                Some(s) => s,
                None => &record.args().to_string(),
            };
            self.app
                .emit(CALLBACK_EVENT, content)
                .map_err(|e| anyhow::anyhow!(e))
        }

        fn flush(&self) {}
    }

    pub fn log_config(handle: AppHandle, level: LevelFilter) -> anyhow::Result<Config> {
        let file = Appender::builder().build(
            FILE_APPENDER_NAME,
            Box::new(
                FileAppender::builder()
                    .build(LOG_FILE_NAME)
                    .context("build file appender")?,
            ),
        );
        let callback =
            Appender::builder().build(CALLBACK_APPENDER_NAME, Box::new(Logger::new(handle)));
        let root = Root::builder()
            .appenders([FILE_APPENDER_NAME, CALLBACK_APPENDER_NAME])
            .build(level);

        Config::builder()
            .appenders([callback, file])
            .build(root)
            .context("build log4rs config")
    }
}
