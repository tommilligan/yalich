use std::collections::HashSet;
use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;

use anyhow::{Context, Result};
use log::{error, info};
use reqwest::blocking::ClientBuilder;
use serde::de::DeserializeOwned;
use structopt::StructOpt;

use yalich::{
    core::{Config, DependencyNames, Resolve},
    github::{self, Github},
    node::{self, npmjs::NpmJs, packagejson::PackageJson},
    python::{self, pypi::PyPI, pyproject::PyProject},
    rust::{self, cargo::Cargo, cratesio::CratesIo},
};

/// yalich collects license metadata from a variety of dependencies.
#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(long, parse(from_os_str))]
    /// Path to config file.
    pub config: PathBuf,
}

fn load_file(path: &PathBuf) -> Result<String> {
    let mut buffer = String::new();
    let mut file = File::open(path).with_context(|| format!("Loading file {}", path.display()))?;
    file.read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn load_toml_file<T: DeserializeOwned>(path: &PathBuf) -> Result<T> {
    let buffer = load_file(path)?;
    toml::from_str(&buffer).with_context(|| format!("With path {}", path.display()))
}

fn load_json_file<T: DeserializeOwned>(path: &PathBuf) -> Result<T> {
    let buffer = load_file(path)?;
    serde_json::from_str(&buffer).with_context(|| format!("With path {}", path.display()))
}

fn load_pyproject(path: &PathBuf) -> Result<PyProject> {
    load_toml_file(path)
}

fn load_cargo(path: &PathBuf) -> Result<Cargo> {
    load_toml_file(path)
}

fn load_packagejson(path: &PathBuf) -> Result<PackageJson> {
    load_json_file(path)
}

fn load_package_names<'a, T: DependencyNames>(
    manifest_paths: &[PathBuf],
    loader: impl Fn(&PathBuf) -> Result<T>,
) -> Result<Vec<String>> {
    let mut package_names: HashSet<String> = Default::default();
    for manifest_path in manifest_paths {
        info!("Loading manifest {}", manifest_path.display());
        let manifest: T = loader(manifest_path)?;
        for dependency_name in manifest.dependency_names() {
            package_names.insert(dependency_name.to_owned());
        }
    }
    let mut python_packages: Vec<String> = package_names.into_iter().collect();
    python_packages.sort();
    Ok(python_packages)
}

fn run() -> Result<()> {
    let args = Args::from_args();
    let config: Config = load_toml_file(&args.config)?;

    let client = ClientBuilder::new().user_agent(config.user_agent).build()?;
    let mut writer = csv::Writer::from_writer(io::stdout());

    // Setup API clients
    let cratesio = CratesIo::new(&client);
    let pypi = PyPI::new(&client);
    let npmjs = NpmJs::new(&client);
    let github = Github::new(&client);

    // Setup package name resolvers
    let python_resolver = python::Resolver::new(&config.languages.python.overrides, &pypi);
    let rust_resolver = rust::Resolver::new(&config.languages.rust.overrides, &cratesio);
    let node_resolver = node::Resolver::new(&config.languages.node.overrides, &npmjs);
    let github_enricher = github::Enricher::new(&github);

    // Load package names
    let python_packages = load_package_names(&config.languages.python.manifests, load_pyproject)?;
    let rust_packages = load_package_names(&config.languages.rust.manifests, load_cargo)?;
    let node_packages = load_package_names(&config.languages.node.manifests, load_packagejson)?;

    // Fetch metadata
    let dependencies: Vec<_> = python_packages
        .iter()
        .map(|name| python_resolver.resolve(name))
        .chain(rust_packages.iter().map(|name| rust_resolver.resolve(name)))
        .chain(node_packages.iter().map(|name| node_resolver.resolve(name)))
        .collect::<Result<_>>()?;

    // Fallback to Github if required after first pass
    let dependencies: Vec<_> = dependencies
        .into_iter()
        .map(|dependency| github_enricher.enrich(dependency))
        .collect::<Result<_>>()?;

    // Send final dependencies to writer
    for dependency in dependencies {
        writer
            .serialize(dependency)
            .with_context(|| "CSV serialization failed".to_owned())?;
    }

    writer.flush()?;
    Ok(())
}

fn main() {
    env_logger::init();

    if let Err(error) = run() {
        error!("{}", error)
    }
}
