use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;

use anyhow::Result;
use reqwest::blocking::ClientBuilder;
use serde::de::DeserializeOwned;
use structopt::StructOpt;

use yalich::{
    cargo, core::DependencyNames, cratesio, npmjs, packagejson, pypi, pyproject, Dependency,
};

/// yalich collects license metadata from a variety of dependencies.
#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(long = "poetry-config", parse(from_os_str))]
    /// Path to poetry dependencies file.
    pub poetry_config: Vec<PathBuf>,

    #[structopt(long = "cargo-config", parse(from_os_str))]
    /// Path to Cargo.toml files.
    pub cargo_config: Vec<PathBuf>,

    #[structopt(long = "package-json", parse(from_os_str))]
    /// Path to package.json files.
    pub package_json: Vec<PathBuf>,

    #[structopt(long = "user-agent", default_value = "foss-license-collector")]
    /// User agent to send to APIs.
    pub user_agent: String,
}

fn load_file(path: &PathBuf) -> Result<String> {
    let mut buffer = String::new();
    let mut file = File::open(path)?;
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

    let client = ClientBuilder::new().user_agent(args.user_agent).build()?;
    let mut wtr = csv::Writer::from_writer(io::stdout());

    // Load python dependencies and fetch metadata
    for pyproject_path in &args.poetry_config {
        let pyproject: pyproject::PyProject = load_toml_file(pyproject_path)?;
        for dependency_name in pyproject.tool.poetry.sorted_dependency_names() {
            wtr.serialize(Dependency::from(&pypi::get_package(
                &client,
                dependency_name,
            )?))?;
        }
    }

    // Load rust dependencies and fetch metadata
    for cargo_path in &args.cargo_config {
        let cargo: cargo::Cargo = load_toml_file(cargo_path)?;
        for dependency_name in cargo.dependency_names() {
            wtr.serialize(Dependency::from(&cratesio::get_crate(
                &client,
                dependency_name,
            )?))?;
        }
    }

    // Load npm dependencies and fetch metadata
    for package_json_path in &args.package_json {
        let package_json: packagejson::PackageJson = load_json_file(package_json_path)?;
        for dependency_name in package_json.dependency_names() {
            wtr.serialize(Dependency::from(&npmjs::get_package(
                &client,
                dependency_name,
            )?))?;
        }
    }

    wtr.flush()?;
    Ok(())
}
