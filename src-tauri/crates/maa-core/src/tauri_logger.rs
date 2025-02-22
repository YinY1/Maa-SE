use std::io::{self};

use tauri::{AppHandle, Emitter};

const CALLBACK_EVENT: &str = "callback-log";
pub struct Logger {
    app: AppHandle,
}

unsafe impl Send for Logger {}

impl io::Write for Logger {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let content = str::from_utf8(buf)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        match self.app.emit(CALLBACK_EVENT, content) {
            Ok(_) => Ok(buf.len()),
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, e.to_string())),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Logger {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}
