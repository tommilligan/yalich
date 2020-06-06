use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Dependency {
    pub category: &'static str,
    pub name: String,
    pub url: String,
    pub license: Option<String>,
    #[serde(skip_serializing)]
    pub homepage: Option<String>,
}

pub trait DependencyNames {
    fn dependency_names<'a>(&'a self) -> Box<dyn Iterator<Item = &'a str> + 'a>;
}

pub trait FetchDependency<T> {
    fn fetch_dependency(&self, dependency_name: &str) -> Result<T>;
}

#[derive(Deserialize, Debug, Default)]
pub struct DependencyOverride {
    #[serde(default)]
    pub license: Option<String>,
}
pub type DependencyOverrides = HashMap<String, DependencyOverride>;

#[derive(Deserialize, Debug, Default)]
pub struct Language {
    pub manifests: Vec<PathBuf>,
    #[serde(default)]
    pub overrides: DependencyOverrides,
}

#[derive(Deserialize, Debug)]
pub struct Languages {
    #[serde(default)]
    pub python: Language,
    #[serde(default)]
    pub rust: Language,
    #[serde(default)]
    pub node: Language,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub languages: Languages,
    pub user_agent: String,
}

pub trait Resolve {
    fn resolve(&self, name: &str) -> Result<Dependency>;
}
