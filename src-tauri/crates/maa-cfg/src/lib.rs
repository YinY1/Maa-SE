#![feature(if_let_guard)]
#![deny(warnings)]

pub mod settings;
pub mod task;

use std::{env::current_dir, fs::create_dir_all, path::PathBuf, str::FromStr};

use anyhow::{Context, bail};
use dashmap::DashMap;
use itertools::Itertools;
use log::trace;
use serde::Serialize;
use strum::{Display, EnumString};
pub use task::*;
use tokio::{fs, join};

use crate::settings::{AdbSettings, SettingType};

pub const CFG_DIR: &str = "config";
pub const DEFAULT_CFG_PATH: &str = "default";
pub const CFG_SUFFIX: &str = ".json";
pub const DAILY_CFG: &str = "daily";
pub const EXTRA_TASK_CFG: &str = "extra-task";
pub const SETTINGS_CFG: &str = "settings";
pub const TOOL_STORAGE: &str = "tool-storage";
pub const CUSTOMS_CFG: &str = "customs";
pub const CUSTOMS_STORAGE: &str = "custom-storage";
pub const VERSION_JSON: &str = "versions";

/// 存放数据，而非gui配置本身
#[derive(Debug, Display, EnumString)]
pub enum Storage {
    /// 小工具识别结果，包括干员识别和仓库识别等
    Tool(String),
    /// 用户自定义任务名的数据存储
    Custom(String),
}

#[derive(Debug, Display)]
pub enum ConfigType {
    Task(TaskType),
    Storage(Storage),
    Settings(SettingType),
}

impl FromStr for ConfigType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            s if let Ok(s) = s.parse() => Ok(Self::Settings(s)),
            s if let Ok(t) = s.parse() => Ok(Self::Task(t)),
            s if let Ok(s) = s.parse() => Ok(Self::Storage(s)),
            _ => Err(s.to_string()),
        }
    }
}

pub type ConfigValue = serde_json::Value;

#[derive(Serialize, Clone)]
pub struct Config {
    #[serde(skip)]
    path: PathBuf,
    #[serde(flatten)]
    /// 避免async read file时跨线程持有mutex
    cfgs: DashMap<String, ConfigValue>,
}

macro_rules! get_cfg_path {
    ($path:expr, $ident:ident) => {
        $path.join(constcat::concat!($ident, CFG_SUFFIX))
    };
}

impl Config {
    pub async fn load(cfg_group: Option<String>) -> anyhow::Result<Self> {
        let path = current_dir()
            .context("cwd")?
            .join(CFG_DIR)
            .join(cfg_group.unwrap_or(DEFAULT_CFG_PATH.to_string()));

        if let Err(e) = create_dir_all(&path)
            && !matches!(e.kind(), std::io::ErrorKind::AlreadyExists)
        {
            anyhow::bail!(e);
        };
        let (daily, tools, settings, customs, extra) = join!(
            load_json_obj(get_cfg_path!(path, DAILY_CFG)),
            load_json_obj(get_cfg_path!(path, TOOL_STORAGE)),
            load_json_obj(get_cfg_path!(path, SETTINGS_CFG)),
            load_json_obj(get_cfg_path!(path, CUSTOMS_CFG)),
            load_json_obj(get_cfg_path!(path, EXTRA_TASK_CFG)),
        );

        let cfgs = DashMap::new();
        cfgs.insert(DAILY_CFG.to_string(), daily.context("load daily")?);
        cfgs.insert(TOOL_STORAGE.to_string(), tools.context("load tools")?);
        cfgs.insert(SETTINGS_CFG.to_string(), settings.context("load settings")?);
        cfgs.insert(CUSTOMS_CFG.to_string(), customs.context("load customs")?);
        cfgs.insert(EXTRA_TASK_CFG.to_string(), extra.context("load extras")?);

        Ok(Self { path, cfgs })
    }

    /// 更新缓存并写入本地配置文件
    ///
    /// # Parameter
    ///
    /// `cfg_type`: {name}.json
    ///
    /// `contents`: json string
    pub async fn set_and_write(
        &self,
        cfg_type: ConfigType,
        params: Parameters,
    ) -> anyhow::Result<()> {
        trace!("cache config and write");
        match cfg_type {
            ConfigType::Task(TaskType::Daily(t)) => {
                let path = get_cfg_path!(self.path, DAILY_CFG);
                self.set_and_write_impl(DAILY_CFG, path, t.to_string(), params)
                    .await
                    .context("write daily.json")
            }
            ConfigType::Task(TaskType::Extra(t)) => {
                let path = get_cfg_path!(self.path, EXTRA_TASK_CFG);
                self.set_and_write_impl(EXTRA_TASK_CFG, path, t.to_string(), params)
                    .await
                    .context("write extra_task.json")
            }
            ConfigType::Task(TaskType::Custom(t)) => {
                let path = get_cfg_path!(self.path, CUSTOMS_CFG);
                self.set_and_write_impl(CUSTOMS_CFG, path, t, params)
                    .await
                    .context("write customs.json")
            }
            ConfigType::Storage(Storage::Tool(t)) => {
                let path = get_cfg_path!(self.path, TOOL_STORAGE);
                self.set_and_write_impl(TOOL_STORAGE, path, t, params)
                    .await
                    .context("write customs.json")
            }
            ConfigType::Storage(Storage::Custom(t)) => {
                let path = get_cfg_path!(self.path, CUSTOMS_STORAGE);
                self.set_and_write_impl(CUSTOMS_STORAGE, path, t, params)
                    .await
                    .context("write custom-storage.json")
            }
            ConfigType::Settings(s) => {
                let path = get_cfg_path!(self.path, SETTINGS_CFG);
                self.set_and_write_impl(SETTINGS_CFG, path, s.to_string(), params)
                    .await
                    .context("write settings.json")
            }
        }
    }

    async fn set_and_write_impl(
        &self,
        cfg_type: &str,
        path: PathBuf,
        key: String,
        params: Parameters,
    ) -> anyhow::Result<()> {
        let value = serde_json::to_value(params).context("serde params to json value")?;
        let mut target = self
            .cfgs
            .get_mut(cfg_type)
            .ok_or_else(|| anyhow::anyhow!("no such cfg"))?;

        target
            .as_object_mut()
            .expect("must be object")
            .insert(key, value);

        let contents = serde_json::to_string_pretty(target.value()).context("serde json cache")?;
        fs::write(&path, contents)
            .await
            .with_context(|| format!("write config to {path:?}"))
    }

    pub fn available_daily_tasks(&self) -> TaskQueue {
        self.available_tasks_impl(DAILY_CFG)
    }

    fn available_tasks_impl(&self, key: &str) -> TaskQueue {
        self.cfgs
            .get(key)
            .unwrap()
            .as_object()
            .unwrap()
            .iter()
            .filter(|(_, params)| params["enable"].as_bool().unwrap_or_default())
            .sorted_by_key(|(_, params)| params["index"].as_i64().unwrap_or_default())
            .map(|(name, params)| (name.to_string(), params.to_string()))
            .collect()
    }

    pub fn adb_config(&self) -> anyhow::Result<AdbSettings> {
        self.cfgs
            .get(SETTINGS_CFG)
            .unwrap()
            .get(SettingType::Adb.as_ref())
            .map(|c| serde_json::from_str(c.as_str().unwrap_or_default()))
            .unwrap_or_else(|| Ok(AdbSettings::default()))
            .context("parse adb settings")
    }
}

pub async fn load_json_obj(path: PathBuf) -> anyhow::Result<serde_json::Value> {
    let content = match fs::read_to_string(path).await {
        Ok(s) => s,
        Err(e) if matches!(e.kind(), std::io::ErrorKind::NotFound) => {
            return Ok(serde_json::Value::Object(Default::default()));
        }
        Err(e) => bail!("read file error: {}", e),
    };
    match serde_json::from_str(&content) {
        Ok(v) => Ok(v),
        Err(e) if e.is_eof() => Ok(serde_json::Value::Object(Default::default())),
        Err(e) => bail!("parse json error: {}", e),
    }
}
