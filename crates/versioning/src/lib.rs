mod version;
mod version_req;

pub use version::{CompletionVersion, LatestVersion, Versioned};
pub use version_req::VersionReqExt;

pub use semver::{Version, VersionReq};
