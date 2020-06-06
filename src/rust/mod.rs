use anyhow::Result;

use crate::core::{Dependency, DependencyOverrides, FetchDependency, Resolve};

pub mod cargo;
pub mod cratesio;

use cratesio::{Crate, CrateResource, CratesIo};

pub struct Resolver<'a> {
    overrides: &'a DependencyOverrides,
    cratesio: &'a CratesIo<'a>,
}

impl<'a> Resolver<'a> {
    pub fn new(overrides: &'a DependencyOverrides, cratesio: &'a CratesIo) -> Self {
        Self {
            overrides,
            cratesio,
        }
    }
}

impl<'a> Resolve for Resolver<'a> {
    fn resolve(&self, name: &str) -> Result<Dependency> {
        let package = self.cratesio.fetch_dependency(name)?;

        let CrateResource { crate_, versions } = package;

        let license = versions
            .get(0)
            .expect("Rust crate must have at least one version.")
            .license
            .to_owned();

        let url = crate_.url();
        let Crate { name, homepage, .. } = crate_;

        let mut dependency = Dependency {
            category: "rust",
            name,
            url,
            license,
            homepage,
        };

        if let Some(dependency_override) = self.overrides.get(&dependency.name) {
            if let Some(license) = &dependency_override.license {
                dependency.license = Some(license.to_owned());
            };
        };

        Ok(dependency)
    }
}
