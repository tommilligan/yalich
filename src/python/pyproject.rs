use std::convert::AsRef;

use serde_derive::Deserialize;

use crate::core::DependencyNames;

#[derive(Deserialize)]
pub struct Poetry {
    pub dependencies: toml::value::Table,
}


#[derive(Deserialize)]
pub struct Tool {
    pub poetry: Poetry,
}

#[derive(Deserialize)]
pub struct PyProject {
    pub tool: Tool,
}

impl DependencyNames for PyProject {
    fn dependency_names<'a>(&'a self) -> Box<dyn Iterator<Item = &'a str> + 'a> {
        Box::new(
            self.tool.poetry.dependencies
                .keys()
                .filter(|dependency_name| dependency_name.as_str() != "python")
                .map(AsRef::as_ref),
        )
    }
}
