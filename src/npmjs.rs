use std::collections::HashMap;

use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde_derive::Deserialize;
use url::Url;

#[derive(Debug, Deserialize, Clone)]
pub struct LicenseDetails {
    pub r#type: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum License {
    Plain(String),
    Detailed(LicenseDetails),
}

impl License {
    pub fn to_license_name(self) -> String {
        match self {
            License::Plain(string) => string,
            License::Detailed(details) => details.r#type,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum OneOrMany {
    One(License),
    Many(Vec<License>),
}

impl OneOrMany {
    pub fn into_license(self) -> Option<License> {
        match self {
            OneOrMany::One(one) => Some(one.to_owned()),
            OneOrMany::Many(many) => many.get(0).map(|license| license.to_owned()),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Version {
    #[serde(default)]
    pub license: Option<OneOrMany>,
    #[serde(default)]
    pub licenses: Option<OneOrMany>,
}

impl Version {
    pub fn license(self) -> Option<License> {
        let license = match (self.license, self.licenses) {
            (Some(one_or_many), _) => one_or_many.into_license(),
            (_, Some(one_or_many)) => one_or_many.into_license(),
            _ => return None,
        };
        license
    }
}

#[derive(Debug, Deserialize)]
pub struct DistTags {
    pub latest: String,
}

#[derive(Debug, Deserialize)]
pub struct Package {
    #[serde(default)]
    pub name: String,
    #[serde(rename = "dist-tags")]
    pub dist_tags: DistTags,
    pub versions: HashMap<String, Version>,
}

impl Package {
    pub fn url(&self) -> String {
        format!("https://www.npmjs.com/package/{}", self.name)
    }

    pub fn latest_version(&self) -> &Version {
        self.versions
            .get(&self.dist_tags.latest)
            .expect("Latest version has no metadata")
    }
}

pub fn get_package(client: &Client, package_name: &str) -> Result<Package> {
    let url = Url::parse(&format!("https://registry.npmjs.org/{}", package_name))
        .with_context(|| format!("Invalid URL for npm package '{}'.", package_name))?;
    let mut package: Package = client
        .get(url)
        .send()
        .with_context(|| format!("NPM request for '{}' failed.", package_name))?
        .json()
        .with_context(|| format!("JSON deserialization for '{}' failed.", package_name))?;

    if package.name.is_empty() {
        package.name = package_name.to_owned();
    }
    Ok(package)
}
