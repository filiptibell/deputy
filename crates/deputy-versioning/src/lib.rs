pub use semver::{Version, VersionReq};

mod version;
mod version_req;

pub mod util;

pub use version::{CompletionVersion, LatestVersion, Versioned};
pub use version_req::VersionReqExt;
