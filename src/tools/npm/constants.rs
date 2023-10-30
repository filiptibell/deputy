use std::collections::HashMap;

use once_cell::sync::Lazy;

static TOP_NPM_PACKAGES: &str = include_str!("../../../assets/top-npm-packages.txt");

pub type PackageName = &'static str;

#[derive(Debug, Clone, Copy)]
pub struct PackageInfo {
    pub name: PackageName,
    pub rank: usize,
}

static GROUPED_PACKAGES: Lazy<HashMap<char, HashMap<PackageName, PackageInfo>>> = Lazy::new(|| {
    let package_infos = TOP_NPM_PACKAGES
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            if !line.trim().is_empty() {
                Some(PackageInfo {
                    name: line.trim(),
                    rank: index,
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let mut packages_grouped_by_first_char = HashMap::new();
    for package_info in package_infos {
        let first_char = package_info
            .name
            .chars()
            .next()
            .expect("empty package name");
        let group = packages_grouped_by_first_char
            .entry(first_char)
            .or_insert_with(HashMap::new);
        group.insert(package_info.name, package_info);
    }

    packages_grouped_by_first_char
});

static TOP_PACKAGES: Lazy<Vec<PackageInfo>> = Lazy::new(|| {
    let mut package_infos = TOP_NPM_PACKAGES
        .lines()
        .enumerate()
        .take(256)
        .filter_map(|(index, line)| {
            if !line.trim().is_empty() {
                Some(PackageInfo {
                    name: line.trim(),
                    rank: index,
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    package_infos.sort_unstable_by(|a, b| a.name.partial_cmp(b.name).unwrap());
    package_infos
});

pub fn find_matching_package_infos(s: &str) -> Vec<PackageInfo> {
    if s.is_empty() {
        TOP_PACKAGES.clone()
    } else {
        let first_char = s.chars().next().unwrap();
        let group = match GROUPED_PACKAGES.get(&first_char) {
            None => return Vec::new(),
            Some(g) => g,
        };

        let mut package_infos = group
            .values()
            .filter(|p| p.name.starts_with(s))
            .copied()
            .collect::<Vec<_>>();

        package_infos.sort_unstable_by(|a, b| a.name.partial_cmp(b.name).unwrap());
        package_infos
    }
}
