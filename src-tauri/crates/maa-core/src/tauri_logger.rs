#[cfg(feature = "dynamic-log-level")]
pub use dynamic_log::*;
use tauri::AppHandle;

const CALLBACK_EVENT: &str = "callback-log";

#[derive(Debug)]
pub struct SeAppender {
    app: AppHandle,
    #[cfg(feature = "dynamic-log-level")]
    encoder: Box<dyn log4rs::encode::Encode>,
}

impl SeAppender {
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            #[cfg(feature = "dynamic-log-level")]
            encoder: Box::new(log4rs::encode::pattern::PatternEncoder::new("[{l}] {m}{n}")),
        }
    }
}

#[cfg(not(feature = "dynamic-log-level"))]
impl std::io::Write for SeAppender {
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
    use std::io;

    use anyhow::Context;
    use log::LevelFilter;
    use log4rs::{
        Config,
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
        config::{Appender, Logger, Root},
        encode,
    };
    use tauri::{AppHandle, Emitter};

    use super::{CALLBACK_EVENT, SeAppender};

    const ROLLING_FILE_APPENDER_NAME: &str = "file";
    const CALLBACK_APPENDER_NAME: &str = "callback"; // TODO:换个更合适的名字
    const LOG_FILE_PATH: &str = "debug/maa-se.log";
    const ACTIVE_CRATES_NAMES: &[&str] = &[
        "maa_se_lib",
        "maa_cfg",
        "maa_core",
        "maa_updater",
        "maa_callback",
    ];

    const MAX_LOG_SIZE: u64 = 10_000_000; // 10 MB
    const MAX_LOG_COUNT: u32 = 5;

    struct StringWriter(String);

    impl io::Write for StringWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let s = std::str::from_utf8(buf)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            self.0.push_str(s);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl encode::Write for StringWriter {}

    impl Append for SeAppender {
        fn append(&self, record: &log::Record) -> anyhow::Result<()> {
            let mut content = StringWriter(String::new());
            self.encoder.encode(&mut content, record)?;
            self.app
                .emit(CALLBACK_EVENT, content.0)
                .map_err(|e| anyhow::anyhow!(e))
        }

        fn flush(&self) {}
    }

    // TODO: 优化每次都要重新build一次
    /// setup loggers config,
    /// gui logger in INFO,
    /// file logger in `level`
    pub fn log_config(handle: AppHandle, file_level: LevelFilter) -> anyhow::Result<Config> {
        // rolling file appender
        let trigger = SizeTrigger::new(MAX_LOG_SIZE);
        let roller = FixedWindowRoller::builder()
            .build(constcat::concat!(LOG_FILE_PATH, ".{}"), MAX_LOG_COUNT)
            .unwrap();
        let appender = RollingFileAppender::builder()
            .build(
                LOG_FILE_PATH,
                Box::new(CompoundPolicy::new(Box::new(trigger), Box::new(roller))),
            )
            .unwrap();
        let rolling = Appender::builder().build(ROLLING_FILE_APPENDER_NAME, Box::new(appender));

        let loggers = ACTIVE_CRATES_NAMES.iter().map(|name| {
            Logger::builder()
                .appender(ROLLING_FILE_APPENDER_NAME)
                .additive(false)
                .build(*name, file_level)
        });

        let gui =
            Appender::builder().build(CALLBACK_APPENDER_NAME, Box::new(SeAppender::new(handle)));
        let gui_logger = Logger::builder()
            .appender(CALLBACK_APPENDER_NAME)
            .additive(false)
            .build("maa_callback::msg_handler", LevelFilter::Info);

        let root = Root::builder()
            .appender(ROLLING_FILE_APPENDER_NAME)
            .build(LevelFilter::Error);

        Config::builder()
            .appenders([gui, rolling])
            .loggers(loggers)
            .logger(gui_logger)
            .build(root)
            .context("build log4rs config")
    }
}
