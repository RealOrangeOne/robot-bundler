use lazy_static::lazy_static;
use regex::Regex;
use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde::ser::{Serialize, Serializer};
use std::convert::TryFrom;
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub struct BuildInfo {
    pub commit: String,
    pub branch: Option<String>,
}

impl fmt::Display for BuildInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.branch.as_ref() {
            None => write!(f, "{}", self.commit),
            Some(branch) => write!(f, "{}@{}", self.commit, branch),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct KitVersion {
    pub epoch: u16,
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
    pub dev: bool,
    pub build_info: Option<BuildInfo>,
}

impl fmt::Display for KitVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.dev {
            match &self.build_info {
                Some(build_info) => write!(
                    f,
                    "{}.{}.{}.{}dev:{}",
                    self.epoch, self.major, self.minor, self.patch, build_info
                ),
                None => write!(
                    f,
                    "{}.{}.{}.{}dev",
                    self.epoch, self.major, self.minor, self.patch
                ),
            }
        } else {
            match &self.build_info {
                Some(build_info) => write!(
                    f,
                    "{}.{}.{}.{}:{}",
                    self.epoch, self.major, self.minor, self.patch, build_info
                ),
                None => write!(
                    f,
                    "{}.{}.{}.{}",
                    self.epoch, self.major, self.minor, self.patch
                ),
            }
        }
    }
}

impl TryFrom<&str> for KitVersion {
    type Error = &'static str;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                "^(\\d+)\\.(\\d+)\\.(\\d+)\\.(\\d+)(dev)?(?::([0-9a-f]{5,40})(?:@(\\w+))?)?$"
            )
            .unwrap();
        }
        match RE.captures(val) {
            Some(captures) => {
                let epoch = captures.get(1).unwrap().as_str().parse::<u16>();
                let major = captures.get(2).unwrap().as_str().parse::<u8>();
                let minor = captures.get(3).unwrap().as_str().parse::<u8>();
                let patch = captures.get(4).unwrap().as_str().parse::<u8>();

                // Quickly check that we have parsed them okay, in case they overflow!
                let epoch = match epoch {
                    Ok(e) => e,
                    Err(_) => return Err("Unable to parse version epoch"),
                };
                let major = match major {
                    Ok(e) => e,
                    Err(_) => return Err("Unable to parse version major value"),
                };
                let minor = match minor {
                    Ok(e) => e,
                    Err(_) => return Err("Unable to parse version minor value"),
                };
                let patch = match patch {
                    Ok(e) => e,
                    Err(_) => return Err("Unable to parse version patch value"),
                };

                // Check the dev indicator
                let dev = captures.get(5).is_some();

                let commit = captures.get(6);
                let branch = captures.get(7);

                let build_info = match (commit, branch) {
                    (None, None) => None,
                    (None, Some(_)) => None,
                    (Some(commit), None) => Some(BuildInfo {
                        commit: commit.as_str().to_string(),
                        branch: None,
                    }),
                    (Some(commit), Some(branch)) => Some(BuildInfo {
                        commit: commit.as_str().to_string(),
                        branch: Some(branch.as_str().to_string()),
                    }),
                };

                Ok(KitVersion {
                    epoch,
                    major,
                    minor,
                    patch,
                    dev,
                    build_info,
                })
            }
            None => Err("version was not in valid format."),
        }
    }
}

impl Serialize for KitVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", self);
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for KitVersion {
    fn deserialize<D>(deserializer: D) -> Result<KitVersion, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(KitVersionVisitor)
    }
}

struct KitVersionVisitor;

impl<'de> Visitor<'de> for KitVersionVisitor {
    type Value = KitVersion;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a valid kit version string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match KitVersion::try_from(value) {
            Ok(s) => Ok(s),
            Err(e) => Err(de::Error::custom(e)),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::KitVersion;
    use std::convert::TryFrom;

    #[test]
    fn parse_version_normal() {
        KitVersion::try_from("2021.0.0.1").unwrap();
    }

    #[test]
    fn parse_version_dev() {
        KitVersion::try_from("2021.0.0.1dev").unwrap();
    }

    #[test]
    fn parse_version_hash() {
        KitVersion::try_from("2021.0.0.1:123456").unwrap();
    }

    #[test]
    fn parse_version_dev_hash() {
        KitVersion::try_from("2021.0.0.1dev:123456").unwrap();
    }

    #[test]
    fn parse_version_branch() {
        KitVersion::try_from("2021.0.0.1:123456@master").unwrap();
    }

    #[test]
    fn parse_version_dev_branch() {
        KitVersion::try_from("2021.0.0.1dev:123456@master").unwrap();
    }
}
