use std::{collections::HashMap, path::PathBuf};

use backtrace::Backtrace;
use serde::{Deserialize, Serialize};

use crate::asset_filter::AssetFilterOption;

pub trait AssetFilterError {}

#[derive(Debug)]
pub struct AssetError<E>
where
    E: AssetFilterError,
{
    pub error_type: AssetErrorType<E>,
    pub backtrace: Backtrace,
}

impl<E> AssetError<E>
where
    E: AssetFilterError,
{
    pub fn new(error_type: AssetErrorType<E>) -> Self {
        AssetError {
            error_type,
            backtrace: Backtrace::new(),
        }
    }
}

#[derive(Debug)]
pub enum AssetErrorType<E>
where
    E: AssetFilterError,
{
    IOError(std::io::Error),
    JSONError(serde_json::Error),
    FilterError(E),
    AssetFilterNotFoundError(String),
    AssetNotFoundInManifestError(String),
    AssetPathError(PathBuf),
}

impl<E> From<std::io::Error> for AssetError<E>
where
    E: AssetFilterError,
{
    fn from(err: std::io::Error) -> Self {
        AssetError {
            error_type: AssetErrorType::IOError(err),
            backtrace: Backtrace::new(),
        }
    }
}

impl<E> From<serde_json::Error> for AssetError<E>
where
    E: AssetFilterError,
{
    fn from(err: serde_json::Error) -> Self {
        AssetError {
            error_type: AssetErrorType::JSONError(err),
            backtrace: Backtrace::new(),
        }
    }
}

impl<E> From<E> for AssetError<E>
where
    E: AssetFilterError,
{
    fn from(err: E) -> Self {
        AssetError {
            error_type: AssetErrorType::FilterError(err),
            backtrace: Backtrace::new(),
        }
    }
}

pub type AssetResult<T, E> = Result<T, AssetError<E>>;

/// Asset source.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum AssetSource {
    /// File from source directory.
    File(PathBuf),
    /// Result of processing other assets by filter.
    Filtered(AssetFiltered),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AssetFiltered {
    /// Filter name in filter registry.
    pub filter_name: String,
    /// Input asset names.
    pub input_names: Vec<String>,
    /// Other options passed to filter.
    pub options: HashMap<String, AssetFilterOption>,
}

/// Asset definition.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AssetData {
    /// Base directory path for asset file in internal and output directories. Should be relative and not pointing to directories outside like `..` or `data/../..`.
    pub output_base_path: Option<PathBuf>,
    /// Extension for asset files in internal and output directories.
    pub extension: String,
    /// Asset source definition.
    pub source: AssetSource,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AssetManifest {
    pub assets: HashMap<String, AssetData>,
    pub public_assets: Vec<String>,
}
