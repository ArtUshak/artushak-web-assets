use std::{
    fs::{self, copy, create_dir_all, remove_file},
    path::{Path, PathBuf},
};

use log::debug;
use path_dedot::ParseDot;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    asset_config::AssetConfig,
    asset_filter::AssetFilterRegistry,
    assets::{
        AssetData, AssetError, AssetErrorType, AssetFilterError, AssetManifest, AssetResult,
        AssetSource,
    },
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AssetCacheEntry {
    pub name: String,
    pub data: AssetData,
    pub path: PathBuf,
    pub file_hash: Option<[u8; blake3::OUT_LEN]>,
}

impl AssetCacheEntry {
    pub fn create<E>(
        name: String,
        config: &AssetConfig,
        manifest: &AssetManifest,
        cache_manifest: &mut AssetCacheManifestV1,
        filter_registry: &AssetFilterRegistry<E>,
    ) -> AssetResult<AssetCacheEntry, E>
    where
        E: AssetFilterError,
    {
        let data = manifest
            .assets
            .get(&name)
            .ok_or_else(|| {
                AssetError::new(AssetErrorType::AssetNotFoundInManifestError(name.clone()))
            })?
            .clone();

        let uuid = Uuid::new_v4();

        let output_path: PathBuf = match &data.output_base_path {
            Some(base_path) => base_path
                .clone()
                .join(name.clone() + "-" + &uuid.to_string())
                .with_extension(data.extension.clone()),
            None => Path::new(&(name.clone() + "-" + &uuid.to_string()))
                .with_extension(data.extension.clone()),
        };
        if output_path.has_root() || output_path.parse_dot().unwrap().starts_with("..") {
            return Err(AssetError::new(AssetErrorType::AssetPathError(output_path)));
        }
        let output_full_path = config.internal_directory_path.join(output_path.clone());

        let file_hash = match &data.source {
            AssetSource::File(file_path) => {
                let source_full_path = config.source_directory_path.join(&file_path);

                debug!("Copying {:?} to {:?}", source_full_path, output_full_path);
                if let Some(output_full_path_parent) = output_full_path.parent() {
                    create_dir_all(output_full_path_parent)?;
                }
                copy(&source_full_path, &output_full_path)?;

                let file_bytes = fs::read(&output_full_path)?;
                let file_hash = blake3::hash(file_bytes.as_slice());
                Some(*file_hash.as_bytes())
            }
            AssetSource::Filtered(filtered) => {
                let mut input_full_paths: Vec<PathBuf> =
                    Vec::with_capacity(filtered.input_names.len());
                for input_name in &filtered.input_names {
                    input_full_paths.push(
                        config.internal_directory_path.join(
                            cache_manifest
                                .process(input_name.clone(), config, manifest, filter_registry)?
                                .0
                                .path,
                        ),
                    );
                }

                filter_registry
                    .process_asset_file(
                        filtered.filter_name.clone(),
                        &input_full_paths,
                        &output_full_path,
                        &filtered.options,
                    )
                    .ok_or_else(|| {
                        AssetError::new(AssetErrorType::AssetFilterNotFoundError(
                            filtered.filter_name.clone(),
                        ))
                    })??;

                None
            }
        };

        Ok(AssetCacheEntry {
            name,
            data,
            path: output_path,
            file_hash,
        })
    }

    pub fn update<E>(
        &self,
        config: &AssetConfig,
        manifest: &AssetManifest,
        cache_manifest: &mut AssetCacheManifestV1,
        filter_registry: &AssetFilterRegistry<E>,
    ) -> AssetResult<Option<AssetCacheEntry>, E>
    where
        E: AssetFilterError,
    {
        let new_data = manifest
            .assets
            .get(&self.name)
            .ok_or_else(|| {
                AssetError::new(AssetErrorType::AssetNotFoundInManifestError(
                    self.name.clone(),
                ))
            })?
            .clone();

        let full_path = config.internal_directory_path.join(self.path.clone());

        let need_update: bool;
        if (new_data != self.data) || !full_path.exists() {
            need_update = true;
        } else {
            need_update = match self.data.source.clone() {
                AssetSource::File(path) => {
                    let full_path = config.source_directory_path.join(path);
                    let file_bytes = fs::read(&full_path)?;
                    let file_hash = blake3::hash(file_bytes.as_slice());

                    if let Some(self_file_hash_bytes) = self.file_hash {
                        file_hash.as_bytes() != &self_file_hash_bytes
                    } else {
                        true
                    }
                }
                AssetSource::Filtered(filtered) => {
                    let mut has_updated_inputs = false;
                    for input_name in filtered.input_names {
                        let (_, changed) = cache_manifest.process(
                            input_name,
                            config,
                            manifest,
                            filter_registry,
                        )?;
                        if changed {
                            has_updated_inputs = true;
                            break;
                        }
                    }

                    has_updated_inputs
                }
            }
        }

        if need_update {
            if full_path.exists() {
                remove_file(full_path)?;
            }

            let target_full_path = config.target_directory_path.join(self.path.clone());
            if target_full_path.exists() {
                remove_file(target_full_path)?;
            }

            return AssetCacheEntry::create(
                self.name.clone(),
                config,
                manifest,
                cache_manifest,
                filter_registry,
            )
            .map(Option::Some);
        }

        Ok(None)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum AssetCacheManifestVersioned {
    V1(AssetCacheManifestV1),
}

impl Default for AssetCacheManifestVersioned {
    fn default() -> Self {
        AssetCacheManifestVersioned::V1(AssetCacheManifestV1::default())
    }
}

/// Asset cache manifest. It contains current file paths, data to check if assets are modified, etc.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AssetCacheManifestV1 {
    pub map: std::collections::HashMap<String, AssetCacheEntry>,
}

impl AssetCacheManifestV1 {
    pub fn process<E>(
        &mut self,
        name: String,
        config: &AssetConfig,
        manifest: &AssetManifest,
        filter_registry: &AssetFilterRegistry<E>,
    ) -> AssetResult<(AssetCacheEntry, bool), E>
    where
        E: AssetFilterError,
    {
        let cache_entry_optional = self.map.get(&name).cloned();
        match cache_entry_optional {
            Some(cache_entry) => {
                if let Some(cache_entry_new) =
                    cache_entry.update(config, manifest, self, filter_registry)?
                {
                    self.map.insert(name, cache_entry_new.clone());
                    Ok((cache_entry_new, true))
                } else {
                    Ok((cache_entry, false))
                }
            }
            None => {
                let result =
                    AssetCacheEntry::create(name.clone(), config, manifest, self, filter_registry);
                if let Ok(cache_entry) = &result {
                    self.map.insert(name, cache_entry.clone());
                }
                result.map(|cache_entry| (cache_entry, true))
            }
        }
    }

    pub fn process_public_assets<E>(
        &mut self,
        config: &AssetConfig,
        manifest: &AssetManifest,
        filter_registry: &AssetFilterRegistry<E>,
    ) -> AssetResult<(), E>
    where
        E: AssetFilterError,
    {
        for asset_name in &manifest.public_assets {
            let (cache_entry, _) =
                self.process(asset_name.clone(), config, manifest, filter_registry)?;

            let source_full_path = config
                .internal_directory_path
                .join(cache_entry.path.clone());
            let output_full_path = config.target_directory_path.join(&cache_entry.path);
            if cache_entry.path.has_root()
                || cache_entry.path.parse_dot().unwrap().starts_with("..")
            {
                return Err(AssetError::new(AssetErrorType::AssetPathError(
                    cache_entry.path,
                )));
            }
            debug!("Copying {:?} to {:?}", source_full_path, output_full_path);
            if let Some(output_full_path_parent) = output_full_path.parent() {
                create_dir_all(output_full_path_parent)?;
            }
            copy(&source_full_path, &output_full_path)?;
        }

        Ok(())
    }

    pub fn get_entry(&self, name: &str) -> Option<AssetCacheEntry> {
        return self.map.get(name).cloned();
    }
}

pub type AssetCacheManifest = AssetCacheManifestV1;
