use serde_derive::Serialize;

pub mod cargo;
pub mod core;
pub mod cratesio;
pub mod npmjs;
pub mod packagejson;
pub mod pypi;
pub mod pyproject;

#[derive(Serialize)]
pub struct Dependency<'a> {
    category: &'a str,
    name: String,
    url: String,
    license: String,
}

impl<'a> From<pypi::Package> for Dependency<'a> {
    fn from(package: pypi::Package) -> Self {
        Dependency {
            category: "python",
            name: package.info.name,
            url: package.info.project_url,
            license: package.info.license,
        }
    }
}

impl<'a> From<cratesio::CrateResource> for Dependency<'a> {
    fn from(crate_resource: cratesio::CrateResource) -> Self {
        let cratesio::CrateResource { crate_, versions } = crate_resource;
        let url = crate_.url();
        Dependency {
            category: "rust",
            name: crate_.name,
            url,
            license: versions[0].license.clone(),
        }
    }
}

impl<'a> From<npmjs::Package> for Dependency<'a> {
    fn from(package: npmjs::Package) -> Self {
        let url = package.url();
        let name = package.name.clone();
        let version = package.latest_version().to_owned();
        let license = version
            .get_license()
            .map(|license| license.name().to_owned())
            .unwrap_or_else(|| String::from("unknown"));
        Dependency {
            category: "node",
            name,
            url,
            license,
        }
    }
}
