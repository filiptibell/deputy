use std::{collections::HashMap, fmt, ops::Range};

use serde::Deserialize;
use toml::{Spanned, Value as TomlValue};
use tracing::error;

const INVALID_DEPS_PANIC_MSG: &str = "Invalid deps should have been filtered out";

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(transparent)]
pub struct ManifestDependency(Spanned<TomlValue>);

// There's literally no way to make this enum work with serde
// and properly keeping spans. I've tried all of the attributes,
// making proxy structs, manual implementations of deserialize,
// splitting string into parts and separately calling toml::from_str,
// implementing several different kinds of visitors, nothing works.
// Will have to go back to the old lexer implementation and try
// to make some kind of reasonable dom we can traverse instead.
// pub enum ManifestDependency {
//     Plain(Spanned<String>),
//     Struct { version: Spanned<String> },
// }

impl ManifestDependency {
    pub fn span(&self) -> Range<usize> {
        self.0.span()
    }
}

impl fmt::Display for ManifestDependency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.as_ref() {
            TomlValue::String(s) => s.fmt(f),
            TomlValue::Table(t) => match t.get("version").expect(INVALID_DEPS_PANIC_MSG) {
                TomlValue::String(s) => s.fmt(f),
                _ => unreachable!("{INVALID_DEPS_PANIC_MSG}"),
            },
            _ => unreachable!("{INVALID_DEPS_PANIC_MSG}"),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Manifest {
    #[serde(default)]
    pub dependencies: HashMap<String, ManifestDependency>,
    #[serde(default, alias = "dev-dependencies")]
    pub dev_dependencies: HashMap<String, ManifestDependency>,
    #[serde(default, alias = "build-dependencies")]
    pub build_dependencies: HashMap<String, ManifestDependency>,
}

impl Manifest {
    pub fn parse(source: impl AsRef<str>) -> Result<Self, toml::de::Error> {
        let result = toml::from_str::<Manifest>(source.as_ref())
            .map(|manifest| manifest.remove_invalid_deps());
        if let Err(e) = &result {
            error!("failed to deserialize cargo manifest - {e}")
        }
        result
    }

    fn remove_invalid_deps(mut self) -> Self {
        let check = |deps: &mut HashMap<String, ManifestDependency>| {
            deps.retain(|_, val| match val.0.as_ref() {
                TomlValue::String(_) => true,
                TomlValue::Table(t) => matches!(t.get("version"), Some(TomlValue::String(_))),
                _ => false,
            });
        };
        check(&mut self.dependencies);
        check(&mut self.dev_dependencies);
        check(&mut self.build_dependencies);
        self
    }
}
