use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde_derive::Deserialize;
use url::Url;

use crate::core::FetchDependency;

#[derive(Debug, Deserialize)]
pub struct Version {
    pub license: String,
}

#[derive(Debug, Deserialize)]
pub struct Crate {
    pub name: String,
}

impl Crate {
    pub fn url(&self) -> String {
        format!("https://crates.io/crates/{}", self.name)
    }
}

#[derive(Debug, Deserialize)]
pub struct CrateResource {
    #[serde(rename = "crate")]
    pub crate_: Crate,
    pub versions: Vec<Version>,
}

pub struct CratesIo<'a> {
    client: &'a Client,
}

impl<'a> CratesIo<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }
}

impl<'a> FetchDependency<CrateResource> for CratesIo<'a> {
    fn fetch_dependency(&self, crate_name: &str) -> Result<CrateResource> {
        let url = Url::parse(&format!("https://crates.io/api/v1/crates/{}", crate_name))
            .with_context(|| format!("Invalid URL for rust crate '{}'.", crate_name))?;
        self.client
            .get(url)
            .send()
            .with_context(|| format!("Crates.io request for '{}' failed.", crate_name))?
            .json()
            .with_context(|| format!("JSON deserialization for '{}' failed.", crate_name))
    }
}
