use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;

use anyhow::{Context, Result};
use reqwest::blocking::ClientBuilder;
use serde::de::DeserializeOwned;
use serde_derive::Deserialize;
use structopt::StructOpt;

use yalich::{
    cargo, core::DependencyNames, cratesio, npmjs, packagejson, pypi, pyproject, Dependency,
};

#[derive(Deserialize, Debug, Default)]
pub struct DependencyOverride {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    license: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
struct Language {
    manifests: Vec<PathBuf>,
    #[serde(default)]
    overrides: HashMap<String, DependencyOverride>,
}

#[derive(Deserialize, Debug)]
struct Languages {
    #[serde(default)]
    python: Language,
    #[serde(default)]
    rust: Language,
    #[serde(default)]
    node: Language,
}

#[derive(Deserialize, Debug)]
struct Config {
    languages: Languages,
    user_agent: String,
}

/// yalich collects license metadata from a variety of dependencies.
#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(long, parse(from_os_str))]
    /// Path to config file.
    pub config: PathBuf,

    #[structopt(long)]
    /// Stop after processing one dependency of each manifest.
    pub debug: bool,
}

fn load_file(path: &PathBuf) -> Result<String> {
    let mut buffer = String::new();
    let mut file = File::open(path).with_context(|| format!("Loading file {}", path.display()))?;
    file.read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn load_toml_file<T: DeserializeOwned>(path: &PathBuf) -> Result<T> {
    let buffer = load_file(path)?;
    let toml = toml::from_str(&buffer)?;
    Ok(toml)
}

fn load_json_file<T: DeserializeOwned>(path: &PathBuf) -> Result<T> {
    let buffer = load_file(path)?;
    let toml = serde_json::from_str(&buffer)?;
    Ok(toml)
}

fn main() -> Result<()> {
    let args = Args::from_args();
    let config: Config = load_toml_file(&args.config)?;

    let client = ClientBuilder::new().user_agent(config.user_agent).build()?;
    let mut wtr = csv::Writer::from_writer(io::stdout());

    // Load python dependencies and fetch metadata
    for pyproject_path in &config.languages.python.manifests {
        let pyproject: pyproject::PyProject = load_toml_file(pyproject_path)?;
        for dependency_name in pyproject.tool.poetry.sorted_dependency_names() {
            wtr.serialize(Dependency::from(&pypi::get_package(
                &client,
                dependency_name,
            )?))?;

            if args.debug {
                break;
            }
        }
    }

    // Load rust dependencies and fetch metadata
    for cargo_path in &config.languages.rust.manifests {
        let cargo: cargo::Cargo = load_toml_file(cargo_path)?;
        for dependency_name in cargo.dependency_names() {
            wtr.serialize(Dependency::from(&cratesio::get_crate(
                &client,
                dependency_name,
            )?))?;

            if args.debug {
                break;
            }
        }
    }

    // Load npm dependencies and fetch metadata
    for package_json_path in &config.languages.node.manifests {
        let package_json: packagejson::PackageJson = load_json_file(package_json_path)?;
        for dependency_name in package_json.dependency_names() {
            wtr.serialize(Dependency::from(&npmjs::get_package(
                &client,
                dependency_name,
            )?))?;

            if args.debug {
                break;
            }
        }
    }

    wtr.flush()?;
    Ok(())
}
