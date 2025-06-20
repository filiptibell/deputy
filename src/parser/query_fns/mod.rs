mod cargo_toml;
mod package_json;

pub use cargo_toml::query_cargo_toml_dependencies;
pub use package_json::query_package_json_dependencies;
