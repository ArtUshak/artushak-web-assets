use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Asset paths configuration.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct AssetConfig {
    /// Directory to copy public assets to.
    pub target_directory_path: PathBuf,
    /// Internal asset storage (should be persistent between pack runs to avoid re-running filter every time).
    pub internal_directory_path: PathBuf,
    /// Directory to get asset sources from.
    pub source_directory_path: PathBuf,
}
