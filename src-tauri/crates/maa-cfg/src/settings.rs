use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumString};

#[derive(Debug, EnumString, Display, AsRefStr)]
pub enum SettingType {
    Adb,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AdbSettings {
    pub path: PathBuf,
    pub address: String,
    pub extra: ExtraAdb,
}

impl Default for AdbSettings {
    fn default() -> Self {
        Self {
            path: PathBuf::from(mumu::DEFAULT_ADB_PATH),
            address: mumu::DEFAULT_ADB_ADDRESS.to_owned(),
            extra: ExtraAdb::MuMuEmulator12(Default::default()),
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize, AsRefStr, Clone)]
pub enum ExtraAdb {
    #[default]
    #[strum(serialize = "")]
    None,
    MuMuEmulator12(mumu::MuMuEmulator12ConnectionExtras),
    // TODO: 雷电
}

pub mod mumu {

    use std::path::PathBuf;

    use serde::{Deserialize, Serialize};
    use serde_json::json;

    pub const DEFAULT_EMULATOR_PATH: &str = "D:\\MuMuPlayer-12.0";
    pub const DEFAULT_ADB_PATH: &str = "D:\\MuMuPlayer-12.0\\shell\\adb.exe";
    pub const DEFAULT_ADB_ADDRESS: &str = "127.0.0.1:16384";

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct MuMuEmulator12ConnectionExtras {
        pub enable: bool,
        bridge_index: Option<usize>,
        emulator_path: PathBuf,
    }

    impl Default for MuMuEmulator12ConnectionExtras {
        fn default() -> Self {
            Self {
                enable: true,
                bridge_index: None,
                emulator_path: PathBuf::from(DEFAULT_EMULATOR_PATH),
            }
        }
    }

    impl MuMuEmulator12ConnectionExtras {
        pub fn new(enable: bool, emulator_path: PathBuf, bridge_index: Option<usize>) -> Self {
            Self {
                enable,
                emulator_path,
                bridge_index,
            }
        }

        pub fn config(&self) -> String {
            if !self.enable {
                return json!({}).to_string();
            }

            let mut config = json!( {
                "path": self.emulator_path.to_string_lossy(),
            });
            self.bridge_index.inspect(|index| {
                config["index"] = json!(index);
            });

            config.to_string()
        }
    }
}
