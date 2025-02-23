use std::sync::{LazyLock, Mutex};

use anyhow::Context;
use maa_cfg::{Config, ConfigType};

static CONFIG: LazyLock<Mutex<Option<Config>>> = LazyLock::new(|| Mutex::new(None));

pub fn update_config(cfg_type: ConfigType, contents: &str) -> anyhow::Result<()> {
    let mut lock = CONFIG.lock().unwrap();
    if lock.is_none() {
        let cfg = Config::load(None).context("load default cfg")?;
        lock.replace(cfg);
    }
    lock.as_mut()
        .unwrap()
        .set_and_write(cfg_type, contents)
        .context("set cfg")
}
