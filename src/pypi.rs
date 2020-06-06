use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde_derive::Deserialize;
use url::Url;

use crate::core::FetchDependency;

#[derive(Debug, Deserialize)]
pub struct Info {
    pub license: String,
    pub name: String,
    pub project_url: String,
}

#[derive(Debug, Deserialize)]
pub struct Package {
    pub info: Info,
}

pub struct PyPI<'a> {
    client: &'a Client,
}

impl<'a> PyPI<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }
}

impl<'a> FetchDependency<Package> for PyPI<'a> {
    fn fetch_dependency(&self, package_name: &str) -> Result<Package> {
        let url = Url::parse(&format!("https://pypi.org/pypi/{}/json", package_name))
            .with_context(|| format!("Invalid URL for pypi package '{}'.", package_name))?;
        self.client
            .get(url)
            .send()
            .with_context(|| format!("Pypi request for '{}' failed.", package_name))?
            .json()
            .with_context(|| format!("JSON deserialization for '{}' failed.", package_name))
    }
}
