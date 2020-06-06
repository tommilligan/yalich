use std::collections::HashMap;

use serde_derive::Deserialize;

use crate::core::DependencyNames;

#[derive(Deserialize)]
pub struct PackageJson {
    pub dependencies: HashMap<String, String>,
}

impl DependencyNames for PackageJson {
    fn dependency_names<'a>(&'a self) -> Box<dyn Iterator<Item = &'a str> + 'a> {
        Box::new(
            self.dependencies
                .iter()
                // filter out local package links
                .filter(|(name, spec)| {
                    !(spec.starts_with("../") || name.starts_with("@fortawesome/pro"))
                })
                .map(|(name, _spec)| name.as_ref()),
        )
    }
}
