use zed_extension_api::{self as zed, Result};

use crate::{constants::GITHUB_REPO, platform::PlatformDescriptor};

pub fn find_latest_release(pdesc: &PlatformDescriptor) -> Result<(String, String)> {
    let release = zed::latest_github_release(
        GITHUB_REPO,
        zed::GithubReleaseOptions {
            require_assets: true,
            pre_release: false,
        },
    )?;

    let release_version = release.version.trim().trim_start_matches('v');

    let asset_name = pdesc.release_asset_name(release_version);
    let asset_found = release
        .assets
        .iter()
        .find(|asset| asset.name == asset_name)
        .ok_or_else(|| {
            format!(
                "failed to find release asset!\
                    \n- name: {asset_name}\
                    \n- version: {release_version}",
            )
        })?;

    Ok((
        release_version.to_string(),
        asset_found.download_url.to_string(),
    ))
}
