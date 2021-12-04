use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use log::debug;
use serde::{Deserialize, Serialize};

use crate::assets::{AssetError, AssetFilterError};

/// Options passed to asset filter.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum AssetFilterOption {
    Flag,
    Bool(bool),
    String(String),
    StringList(Vec<String>),
}

/// Return `Some(true)` if option is set and is flag, `Some(false)` if option is not set, `None` if option has other type.
pub fn option_is_flag(option: Option<AssetFilterOption>) -> Option<bool> {
    match option {
        Some(AssetFilterOption::Flag) => Some(true),
        None => Some(false),
        _ => None,
    }
}

/// Return `Some(true)` if option is true, `Some(false)` if option is false, `None` if option has other type.
pub fn get_bool(option: AssetFilterOption) -> Option<bool> {
    match option {
        AssetFilterOption::Bool(value) => Some(value),
        _ => None,
    }
}

/// Return `Some(x)` if option is string `x`, `None` if options has other type.
pub fn get_string(option: AssetFilterOption) -> Option<String> {
    match option {
        AssetFilterOption::String(value) => Some(value),
        _ => None,
    }
}

/// Return `Some(x)` if option is string list `x`, `None` if options has other type.
pub fn get_string_list(option: AssetFilterOption) -> Option<Vec<String>> {
    match option {
        AssetFilterOption::StringList(value) => Some(value),
        _ => None,
    }
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
