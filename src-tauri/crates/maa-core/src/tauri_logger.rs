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
    use std::{ops::Deref, sync::OnceLock};

    use anyhow::Context;
    use log::LevelFilter;
    use log4rs::{
        Config, Handle,
        append::{
            Append,
            rolling_file::{
                RollingFileAppender,
                policy::compound::{
                    CompoundPolicy, roll::fixed_window::FixedWindowRoller,
                    trigger::size::SizeTrigger,
                },
            },
        },
        config::{Appender, Root},
    };
    use tauri::{AppHandle, Emitter};

    use super::{CALLBACK_EVENT, Logger};

    const ROLLING_FILE_APPENDER_NAME: &str = "file";
    const CALLBACK_APPENDER_NAME: &str = "callback";
    const LOG_FILE_PATH: &str = "debug/maa-se.log";

    const MAX_LOG_SIZE: u64 = 10_000_000; // 10 MB
    const MAX_LOG_COUNT: u32 = 5;

    #[derive(Default)]
    pub struct LogHandleState(OnceLock<Handle>);

    impl Deref for LogHandleState {
        type Target = OnceLock<Handle>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

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

    // TODO: 优化每次都要重新build一次
    pub fn log_config(handle: AppHandle, level: LevelFilter) -> anyhow::Result<Config> {
        let trigger = SizeTrigger::new(MAX_LOG_SIZE);
        let roller = FixedWindowRoller::builder()
            .build(&format!("{LOG_FILE_PATH}.{{}}"), MAX_LOG_COUNT) // 保留最多10个备份文件（配合时间清理）
            .unwrap();
        let appender = RollingFileAppender::builder()
            .build(
                LOG_FILE_PATH,
                Box::new(CompoundPolicy::new(Box::new(trigger), Box::new(roller))),
            )
            .unwrap();
        let rolling = Appender::builder().build(ROLLING_FILE_APPENDER_NAME, Box::new(appender));
        let callback =
            Appender::builder().build(CALLBACK_APPENDER_NAME, Box::new(Logger::new(handle)));

        let root = Root::builder()
            .appenders([ROLLING_FILE_APPENDER_NAME, CALLBACK_APPENDER_NAME])
            .build(level);

        Config::builder()
            .appenders([callback, rolling])
            .build(root)
            .context("build log4rs config")
    }
}
