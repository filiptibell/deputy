mod cargo_toml;
mod package_json;
mod wally_toml;

pub use cargo_toml::query_cargo_toml_dependencies;
pub use package_json::query_package_json_dependencies;
pub use wally_toml::query_wally_toml_dependencies;
