use std::collections::HashMap;

use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde_derive::Deserialize;
use url::Url;

use crate::core::FetchDependency;

#[derive(Debug, Deserialize, Clone)]
pub struct LicenseDetails {
    r#type: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum License {
    Plain(String),
    Detailed(LicenseDetails),
}

impl License {
    pub fn name(&self) -> &str {
        match self {
            License::Plain(string) => string,
            License::Detailed(details) => &details.r#type,
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
    pub fn get_license(&self) -> Option<&License> {
        match self {
            OneOrMany::One(one) => Some(one),
            OneOrMany::Many(many) => many.get(0),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Version {
    #[serde(default)]
    pub license: Option<OneOrMany>,
    #[serde(default)]
    pub licenses: Option<OneOrMany>,
    #[serde(default)]
    pub homepage: Option<String>,
}

impl Version {
    pub fn get_license(&self) -> Option<&License> {
        for field in [&self.license, &self.licenses].iter() {
            if let Some(one_or_many) = field {
                return one_or_many.get_license();
            }
        }
        None
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

pub struct NpmJs<'a> {
    client: &'a Client,
}

impl<'a> NpmJs<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }
}

impl<'a> FetchDependency<Package> for NpmJs<'a> {
    fn fetch_dependency(&self, package_name: &str) -> Result<Package> {
        let url = Url::parse(&format!("https://registry.npmjs.org/{}", package_name))
            .with_context(|| format!("Invalid URL for npm package '{}'.", package_name))?;
        let mut package: Package = self
            .client
            .get(url)
            .send()
            .with_context(|| format!("NPM request for '{}' failed.", package_name))?
            .json()
            .with_context(|| format!("JSON deserialization for '{}' failed.", package_name))?;

        // Horray! npm allows a package to have no name in it's metadata!
        if package.name.is_empty() {
            package.name = package_name.to_owned();
        }
        Ok(package)
    }
}
