pub mod asset_config;
pub mod asset_filter;
pub mod assets;
pub mod assets_cache;
mod test;

use std::fs::File;
use std::path::Path;

use log::debug;

use crate::asset_config::AssetConfig;
use crate::asset_filter::AssetFilterRegistry;
use crate::assets::{AssetFilterError, AssetManifest, AssetResult};
use crate::assets_cache::{AssetCacheManifest, AssetCacheManifestVersioned};

/// Load cache manifest from file.
pub fn load_cache_manifest<E>(cache_manifest_path: &Path) -> AssetResult<AssetCacheManifest, E>
where
    E: AssetFilterError,
{
    let cache_manifest: AssetCacheManifestVersioned;
    if cache_manifest_path.exists() {
        let cache_manifest_file = std::fs::File::open(cache_manifest_path)?;
        cache_manifest = serde_json::from_reader(cache_manifest_file)?;
    } else {
        cache_manifest = AssetCacheManifestVersioned::default();
    }

    match cache_manifest {
        AssetCacheManifestVersioned::V1(cache_manifest_v1) => Ok(cache_manifest_v1),
    }
}

/// Process asset manifest and asset cache manifest stored in files. Generate new asset versions if needed.
pub fn pack<E>(
    manifest_path: &Path,
    cache_manifest_path: &Path,
    config: &AssetConfig,
    filter_registry: &AssetFilterRegistry<E>,
) -> AssetResult<(), E>
where
    E: AssetFilterError,
{
    let manifest: AssetManifest;
    {
        let manifest_file = File::open(manifest_path)?;
        manifest = serde_json::from_reader(manifest_file)?;
    }

    let cache_manifest: AssetCacheManifestVersioned;
    if cache_manifest_path.exists() {
        let cache_manifest_file = std::fs::File::open(cache_manifest_path)?;
        cache_manifest = serde_json::from_reader(cache_manifest_file)?;
    } else {
        cache_manifest = AssetCacheManifestVersioned::default();
    }

    match cache_manifest {
        AssetCacheManifestVersioned::V1(mut cache_manifest_v1) => {
            debug!("Processing assets...");

            let result =
                cache_manifest_v1.process_public_assets(config, &manifest, filter_registry);

            if result.is_ok() {
                debug!("Assets were processed");
            }

            {
                let cache_manifest_file = std::fs::File::create(cache_manifest_path)?;
                serde_json::to_writer(
                    cache_manifest_file,
                    &AssetCacheManifestVersioned::V1(cache_manifest_v1),
                )?;
            }

            result
        }
    }
}
