use std::{
    str::FromStr,
    sync::{Arc, OnceLock},
};

use crate::shared::CompletionMap;

/**
    A statically stored package from the awesome-go curated list.

    Stored in a text file as:

    ```
    github.com/user/repo:Display Name:"description"
    ```

    Note that unlike other statically embedded package lists,
    this is unordered - whereas Cargo etc order by popularity.
*/
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GoPackage {
    pub path: Arc<str>,
    pub name: Arc<str>,
    pub description: Arc<str>,
}

impl FromStr for GoPackage {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((path, rest)) = s.split_once(':') else {
            return Err("missing path".to_string());
        };
        let Some((name, rest)) = rest.split_once(':') else {
            return Err("missing name".to_string());
        };
        let description = rest
            .strip_prefix('"')
            .ok_or_else(|| "unquoted description".to_string())?
            .strip_suffix('"')
            .ok_or_else(|| "unquoted description".to_string())?;
        Ok(Self {
            path: path.into(),
            name: name.into(),
            description: description.into(),
        })
    }
}

impl AsRef<str> for GoPackage {
    fn as_ref(&self) -> &str {
        self.path.as_ref()
    }
}

/*
    We bundle the awesome-go curated package list in a text file,
    and pre-compute them here for fast autocomplete - see the
    implementation for `CompletionMap` for more details on this.
*/

static TOP_PACKAGES_GO: &str = include_str!("../../assets/top-go-packages.txt");
static TOP_PACKAGES: OnceLock<CompletionMap<GoPackage>> = OnceLock::new();

pub fn top_go_packages_prefixed(prefix: &str, limit: usize) -> Vec<&GoPackage> {
    let top = TOP_PACKAGES.get_or_init(|| {
        TOP_PACKAGES_GO
            .lines()
            .map(|s| s.parse().unwrap())
            .collect::<CompletionMap<_>>()
    });

    top.iter(prefix).take(limit).collect()
}
