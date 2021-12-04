use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use log::debug;
use serde::{Deserialize, Serialize};

use crate::assets::{AssetError, AssetFilterError};

/// Options passed to asset filter.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum AssetFilterOption {
    Flag,
    Bool(bool),
    String(String),
    StringList(Vec<String>),
}

/// Trait for filters that process assets.
pub trait AssetFilter<E>
where
    E: AssetFilterError,
{
    /// Process asset: take input files and write output to output file.
    fn process_asset_file(
        &self,
        input_file_paths: &[PathBuf],
        output_file_path: &Path,
        options: &HashMap<String, AssetFilterOption>,
    ) -> Result<(), AssetError<E>>;
}

pub struct AssetFilterRegistry<E> {
    filters: HashMap<String, Box<dyn AssetFilter<E>>>,
}

impl<E> AssetFilterRegistry<E> {
    /// Create asset filter registry from HashMap.
    pub fn new(filters: HashMap<String, Box<dyn AssetFilter<E>>>) -> AssetFilterRegistry<E> {
        AssetFilterRegistry { filters }
    }

    /// Process assets by filter with name filter_name.
    pub fn process_asset_file(
        &self,
        filter_name: String,
        input_file_paths: &[PathBuf],
        output_file_path: &Path,
        options: &HashMap<String, AssetFilterOption>,
    ) -> Option<Result<(), AssetError<E>>>
    where
        E: AssetFilterError,
    {
        debug!(
            "Processing files {:?} to output file {:?} by filter {}",
            input_file_paths, output_file_path, filter_name
        );

        self.filters
            .get(&filter_name)
            .map(|filter| filter.process_asset_file(input_file_paths, output_file_path, options))
    }
}
