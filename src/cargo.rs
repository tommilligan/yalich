use serde_derive::Deserialize;

use crate::core::DependencyNames;

#[derive(Deserialize)]
pub struct Cargo {
    pub dependencies: toml::value::Table,
}

impl DependencyNames for Cargo {
    fn dependency_names<'a>(&'a self) -> Box<dyn Iterator<Item = &'a str> + 'a> {
        Box::new(
            self.dependencies
                .iter()
                .filter(|(_crate_name, crate_spec)| match crate_spec {
                    toml::value::Value::String(_) => true,
                    // Filter out local dependencies with the path property
                    toml::value::Value::Table(crate_spec_table) => {
                        crate_spec_table.get("path").is_none()
                    }
                    _ => false,
                })
                .map(|(crate_name, _crate_spec)| crate_name.as_ref()),
        )
    }
}
