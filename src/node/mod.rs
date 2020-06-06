use anyhow::Result;

use crate::core::{Dependency, DependencyOverrides, FetchDependency, Resolve};

pub mod npmjs;
pub mod packagejson;

use npmjs::{NpmJs, Package};

pub struct Resolver<'a> {
    overrides: &'a DependencyOverrides,
    npmjs: &'a NpmJs<'a>,
}

impl<'a> Resolver<'a> {
    pub fn new(overrides: &'a DependencyOverrides, npmjs: &'a NpmJs) -> Self {
        Self { overrides, npmjs }
    }
}

impl<'a> Resolve for Resolver<'a> {
    fn resolve(&self, name: &str) -> Result<Dependency> {
        let package = self.npmjs.fetch_dependency(name)?;
        let url = package.url();
        let version = package.latest_version();
        let license = version
            .get_license()
            .map(|license| license.name().to_owned());

        let homepage = version.homepage.to_owned();
        let Package { name, .. } = package;

        let mut dependency = Dependency {
            category: "node",
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
