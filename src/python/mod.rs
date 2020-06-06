use anyhow::Result;

use crate::core::{Dependency, DependencyOverrides, FetchDependency, Resolve};

pub mod pypi;
pub mod pyproject;

use pypi::{Info, Package, PyPI};

pub struct Resolver<'a> {
    overrides: &'a DependencyOverrides,
    pypi: &'a PyPI<'a>,
}

impl<'a> Resolver<'a> {
    pub fn new(overrides: &'a DependencyOverrides, pypi: &'a PyPI) -> Self {
        Self { overrides, pypi }
    }
}

impl<'a> Resolve for Resolver<'a> {
    fn resolve(&self, name: &str) -> Result<Dependency> {
        let Package { info } = self.pypi.fetch_dependency(name)?;
        let Info {
            name,
            project_url,
            license,
            home_page,
        } = info;

        let mut dependency = Dependency {
            category: "python",
            name,
            url: project_url,
            license: if license.is_empty() {
                None
            } else {
                Some(license)
            },
            homepage: Some(home_page),
        };

        if let Some(dependency_override) = self.overrides.get(&dependency.name) {
            if let Some(license) = &dependency_override.license {
                dependency.license = Some(license.to_owned());
            };
        };

        Ok(dependency)
    }
}
