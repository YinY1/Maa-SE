use std::{env::current_dir, fs};

use anyhow::{Context, bail};
use chrono::NaiveDateTime;
use semver::Version;
use serde::{Deserialize, Serialize};

pub const CLIENT_VERSION_JSON: &str = "client_version.json";
pub const RESOURCE_VERSION_JSON: &str = "version.json";
pub const RESOURCE_TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M:%S%.3f";

#[derive(Deserialize)]
#[serde(rename_all(deserialize = "lowercase"))]
pub enum ClientVersionRequest {
    Nightly,
    Beta,
    Stable,
}

impl ClientVersionRequest {
    pub fn to_version(self, version: String) -> ClientVersion {
        match self {
            ClientVersionRequest::Nightly => ClientVersion::Nightly(version),
            ClientVersionRequest::Beta => ClientVersion::Beta(version),
            ClientVersionRequest::Stable => ClientVersion::Stable(version),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum ClientVersion {
    /// 内测
    Nightly(String),
    /// 公测
    Beta(String),
    /// 稳定
    Stable(String),
    Unknown,
}

impl ClientVersion {
    pub fn version(&self) -> Option<&str> {
        match self {
            ClientVersion::Nightly(s) | ClientVersion::Beta(s) | ClientVersion::Stable(s) => {
                Some(s)
            }
            ClientVersion::Unknown => None,
        }
    }

    pub fn semver(&self) -> anyhow::Result<Version> {
        match self.version() {
            Some(v) => Version::parse(v.trim_start_matches('v')).context("parser semver"),
            None => bail!("unknown version"),
        }
    }
}

impl ClientVersion {
    pub fn load() -> anyhow::Result<Self> {
        let path = current_dir().context("cwd")?.join(CLIENT_VERSION_JSON);
        let version = match fs::read_to_string(path) {
            Ok(ver) => serde_json::from_str(&ver)
                .with_context(|| constcat::concat!("load ", CLIENT_VERSION_JSON))?,
            Err(e) if matches!(e.kind(), std::io::ErrorKind::NotFound) => ClientVersion::Unknown,
            Err(e) => anyhow::bail!("cannot read version: {e}"),
        };
        Ok(version)
    }

    pub fn write(&self) -> anyhow::Result<()> {
        let path = current_dir().context("cwd")?.join(CLIENT_VERSION_JSON);
        let contents = serde_json::to_string(&self).context("serialize client version")?;
        fs::write(path, contents).context("write client json")
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Activity {
    name: String,
    time: u64,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Gacha {
    pool: String,
    time: u64,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ResourceVersion {
    activity: Activity,
    gacha: Gacha,
    pub last_updated: String,
}

impl ResourceVersion {
    pub fn load() -> anyhow::Result<Self> {
        let path = current_dir()
            .context("cwd")?
            .join("resource")
            .join(RESOURCE_VERSION_JSON);
        let file = match fs::File::options().read(true).open(path) {
            Ok(f) => f,
            Err(e) if matches!(e.kind(), std::io::ErrorKind::NotFound) => {
                return Ok(Self::default());
            }
            Err(e) => bail!(e),
        };

        serde_json::from_reader(file).context("load resource version")
    }

    pub fn reload(&mut self) -> anyhow::Result<()> {
        *self = Self::load()?;
        Ok(())
    }

    pub fn exists(&self) -> bool {
        !self.last_updated.is_empty()
    }

    pub fn timestamp(&self) -> anyhow::Result<NaiveDateTime> {
        NaiveDateTime::parse_from_str(&self.last_updated, RESOURCE_TIMESTAMP_FORMAT)
            .context("parse last_updated")
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Versions {
    pub client: ClientVersion,
    pub resource: ResourceVersion,
}

impl Versions {
    pub fn load() -> anyhow::Result<Self> {
        Ok(Self {
            client: ClientVersion::load()?,
            resource: ResourceVersion::load()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ClientVersion;

    #[test]
    fn version_ser() {
        let nightly = ClientVersion::Nightly("v5.14.0-beta.3.d026.ga1d49556d".to_string());
        let beta = ClientVersion::Beta("v5.14.0-beta.3".to_string());
        let stable = ClientVersion::Stable("v5.13.1".to_string());

        assert_eq!(
            "{\"Nightly\":\"v5.14.0-beta.3.d026.ga1d49556d\"}",
            &serde_json::to_string(&nightly).unwrap()
        );
        assert_eq!(
            "{\"Beta\":\"v5.14.0-beta.3\"}",
            &serde_json::to_string(&beta).unwrap()
        );
        assert_eq!(
            "{\"Stable\":\"v5.13.1\"}",
            &serde_json::to_string(&stable).unwrap()
        );
        assert_eq!(
            "\"Unknown\"",
            &serde_json::to_string(&ClientVersion::Unknown).unwrap()
        );
    }

    #[test]
    fn version_de() {
        let nightly = ClientVersion::Nightly("v5.14.0-beta.3.d026.ga1d49556d".to_string());
        let beta = ClientVersion::Beta("v5.14.0-beta.3".to_string());
        let stable = ClientVersion::Stable("v5.13.1".to_string());

        assert_eq!(
            serde_json::from_str::<ClientVersion>(
                "{\"Nightly\":\"v5.14.0-beta.3.d026.ga1d49556d\"}"
            )
            .unwrap(),
            nightly
        );
        assert_eq!(
            serde_json::from_str::<ClientVersion>("{\"Beta\":\"v5.14.0-beta.3\"}").unwrap(),
            beta
        );
        assert_eq!(
            serde_json::from_str::<ClientVersion>("{\"Stable\":\"v5.13.1\"}").unwrap(),
            stable
        );
        assert_eq!(
            serde_json::from_str::<ClientVersion>("\"Unknown\"").unwrap(),
            ClientVersion::Unknown
        );
    }

    #[test]
    fn semver() {
        let nightly1 = ClientVersion::Nightly("v5.14.0-beta.3.d026.ga1d49556d".to_string());
        let nightly2 = ClientVersion::Nightly("v5.14.0-beta.3.d030.g82b63a0c3".to_string());
        let beta = ClientVersion::Beta("v5.14.0-beta.3".to_string());
        let stable = ClientVersion::Stable("v5.13.1".to_string());

        let nightly1 = nightly1.semver().unwrap();
        let nightly2 = nightly2.semver().unwrap();
        let beta = beta.semver().unwrap();
        let stable = stable.semver().unwrap();

        assert!(nightly1 < nightly2);
        assert!(nightly1 > beta);
        assert!(beta > stable);
    }
}
