use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AssetConfig {
    /// Directory to copy public assets to.
    pub target_directory_path: PathBuf,
    /// Internal asset storage (should be persistent between pack runs??).
    pub internal_directory_path: PathBuf,
    /// Directory to get asset sources from.
    pub source_directory_path: PathBuf,
}
