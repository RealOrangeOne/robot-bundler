use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;

use crate::kit_version::KitVersion;

#[derive(Clone, Deserialize, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BundleVersionSection {
    pub version: String
}

#[derive(Clone, Deserialize, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct KitInformationSection {
    pub name: String,
    pub version: KitVersion,
}

#[derive(Clone, Deserialize, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WiFiInformationSection {
    pub ssid: String,
    pub psk: String,
    pub enabled: bool,
    pub region: String,
}


#[derive(Clone, Deserialize, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BundleInformationSchema {
    pub bundle: BundleVersionSection,
    pub kit: KitInformationSection,
    pub wifi: WiFiInformationSection,
}

impl BundleInformationSchema {
    pub fn load(filename: &str) -> Result<BundleInformationSchema, Box<dyn Error>> {
        let contents = fs::read_to_string(filename)?;
        let info: BundleInformationSchema = toml::from_str(&contents)?;
        Ok(info)
    }

    pub fn to_string(&self) -> String {
        toml::to_string(&self).unwrap()
    }
}

#[cfg(test)]
mod tests {

    use super::BundleInformationSchema;

    #[test]
    fn load_bundle_info_from_file() {
        let info = BundleInformationSchema::load("example-bundle.toml").unwrap();
        assert_eq!(info.bundle.version, "2.0.0");
        assert_eq!(info.kit.name, "Student Robotics");
        assert_eq!(info.kit.version.epoch, 2022);
        assert_eq!(info.kit.version.major, 1);
        assert_eq!(info.kit.version.minor, 4);
        assert_eq!(info.kit.version.patch, 0);
        assert!(!info.kit.version.dev);
        assert_eq!(info.wifi.ssid, "robot-ABC");
        assert_eq!(info.wifi.psk, "beeeeees");
        assert_eq!(info.wifi.region, "GB");
        assert!(info.wifi.enabled);
        println!("{}", info.to_string())
    }
}