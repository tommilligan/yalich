pub mod core;
pub mod github;
pub mod node;
pub mod python;
pub mod rust;

// impl<'a> From<&'a cratesio::CrateResource> for Dependency<'a> {
//     fn from(crate_resource: &'a cratesio::CrateResource) -> Self {
//         let cratesio::CrateResource { crate_, versions } = crate_resource;
//         Dependency {
//             category: "rust",
//             name: &crate_.name,
//             url: crate_.url(),
//             license: &versions
//                 .get(0)
//                 .expect("Rust crate must have at least one version.")
//                 .license,
//         }
//     }
// }
//
// impl<'a> From<&'a npmjs::Package> for Dependency<'a> {
//     fn from(package: &'a npmjs::Package) -> Self {
//         let version = package.latest_version();
//         let license = version
//             .get_license()
//             .map(|license| license.name())
//             .unwrap_or_else(|| "unknown");
//         Dependency {
//             category: "node",
//             name: &package.name,
//             url: package.url(),
//             license,
//         }
//     }
// }
