use std::{
    str::FromStr,
    sync::{Arc, OnceLock},
};

use crate::shared::CompletionMap;

/**
    A statically stored package from the `PyPI` registry.

    Stored in a text file as:

    ```
    name
    other-name
    some-other-name
    ```

    Where the order determines the ranking.
*/
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PyPiPackage {
    pub name: Arc<str>,
    pub ranking: u64,
}

impl FromStr for PyPiPackage {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            name: s.trim().to_string().into(),
            ranking: 0,
        })
    }
}

impl AsRef<str> for PyPiPackage {
    fn as_ref(&self) -> &str {
        self.name.as_ref()
    }
}

/*
    We bundle about 10,000 top PyPI packages in a text file,
    and pre-compute them here for fast autocomplete - see the
    implementation for `PrefixOrderedMap` for more details on this.
*/

static TOP_PACKAGES_PYPI: &str = include_str!("../../assets/top-pypi-packages.txt");
static TOP_PACKAGES: OnceLock<CompletionMap<PyPiPackage>> = OnceLock::new();

pub fn top_pypi_packages_prefixed(prefix: &str, limit: usize) -> Vec<&PyPiPackage> {
    let top = TOP_PACKAGES.get_or_init(|| {
        TOP_PACKAGES_PYPI
            .lines()
            .map(|s| s.parse().unwrap())
            .collect::<CompletionMap<_>>()
    });

    top.iter(prefix).take(limit).collect()
}
