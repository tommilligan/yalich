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
    name: &'a str,
    url: String,
    license: &'a str,
}

impl<'a> From<&'a pypi::Package> for Dependency<'a> {
    fn from(package: &'a pypi::Package) -> Self {
        Dependency {
            category: "python",
            name: &package.info.name,
            url: package.info.project_url.clone(),
            license: &package.info.license,
        }
    }
}

impl<'a> From<&'a cratesio::CrateResource> for Dependency<'a> {
    fn from(crate_resource: &'a cratesio::CrateResource) -> Self {
        let cratesio::CrateResource { crate_, versions } = crate_resource;
        Dependency {
            category: "rust",
            name: &crate_.name,
            url: crate_.url(),
            license: &versions
                .get(0)
                .expect("Rust crate must have at least one version.")
                .license,
        }
    }
}

impl<'a> From<&'a npmjs::Package> for Dependency<'a> {
    fn from(package: &'a npmjs::Package) -> Self {
        let version = package.latest_version();
        let license = version
            .get_license()
            .map(|license| license.name())
            .unwrap_or_else(|| "unknown");
        Dependency {
            category: "node",
            name: &package.name,
            url: package.url(),
            license,
        }
    }
}
