use anyhow::Result;

pub trait DependencyNames {
    fn dependency_names<'a>(&'a self) -> Box<dyn Iterator<Item = &'a str> + 'a>;

    fn sorted_dependency_names(&self) -> Vec<&str> {
        let mut names: Vec<_> = self.dependency_names().collect();
        names.sort();
        names
    }
}

pub trait FetchDependency<T> {
    fn fetch_dependency(&self, dependency_name: &str) -> Result<T>;
}
