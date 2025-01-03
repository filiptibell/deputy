use std::{
    str::FromStr,
    sync::{Arc, OnceLock},
};

use super::PrefixOrderedMap;

/**
    A statically stored package from the crates.io index.

    Stored in a text file as:

    ```
    name:downloads:"description"
    ```
*/
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CratesIoPackage {
    pub name: Arc<str>,
    pub downloads: u64,
    pub description: Arc<str>,
}

impl FromStr for CratesIoPackage {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((name, rest)) = s.split_once(':') else {
            return Err("missing name".to_string());
        };
        let Some((downloads, rest)) = rest.split_once(':') else {
            return Err("missing downloads".to_string());
        };
        let description = rest
            .strip_prefix('"')
            .ok_or_else(|| "unquoted description".to_string())?
            .strip_suffix('"')
            .ok_or_else(|| "unquoted description".to_string())?;
        Ok(Self {
            name: name.into(),
            downloads: downloads.parse().unwrap(),
            description: description.into(),
        })
    }
}

impl AsRef<str> for CratesIoPackage {
    fn as_ref(&self) -> &str {
        self.name.as_ref()
    }
}

/*
    We bundle about 10,000 top crates.io packages in a text file,
    and pre-compute them here for fast autocomplete - see the
    implementation for `PrefixMap` for more details on this.
*/

static TOP_PACKAGES_CRATES_IO: &str = include_str!("../../../assets/top-crates-io-packages.txt");
static TOP_PACKAGES: OnceLock<PrefixOrderedMap<CratesIoPackage>> = OnceLock::new();

pub fn top_crates_io_packages_prefixed(prefix: &str, limit: usize) -> Vec<&CratesIoPackage> {
    let top = TOP_PACKAGES.get_or_init(|| {
        TOP_PACKAGES_CRATES_IO
            .lines()
            .map(|s| s.parse().unwrap())
            .collect::<PrefixOrderedMap<_>>()
    });

    top.get(prefix)
        .iter()
        .filter(|p| p.name.starts_with(prefix))
        .take(limit)
        .collect()
}
