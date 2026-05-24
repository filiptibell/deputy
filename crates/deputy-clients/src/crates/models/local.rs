use std::path::PathBuf;

use deputy_versioning::Version;

#[derive(Debug, Clone)]
pub struct LocalMetadata {
    pub version: Option<Version>,
    pub features: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct WorkspacePackageMetadata {
    pub dependencies: Vec<WorkspaceDependencyMetadata>,
}

impl WorkspacePackageMetadata {
    #[must_use]
    pub fn dependency(&self, name: &str) -> Option<&WorkspaceDependencyMetadata> {
        self.dependencies
            .iter()
            .find(|dep| dep.manifest_name == name)
    }
}

#[derive(Debug, Clone)]
pub struct WorkspaceDependencyMetadata {
    pub name: String,
    pub manifest_name: String,
    pub req: String,
    pub source: Option<String>,
    pub features: Vec<String>,
    pub path: Option<PathBuf>,
}

impl WorkspaceDependencyMetadata {
    #[must_use]
    pub fn is_git(&self) -> bool {
        self.source
            .as_ref()
            .is_some_and(|source| source.starts_with("git+"))
    }

    #[must_use]
    pub fn is_path(&self) -> bool {
        self.path.is_some()
    }
}
