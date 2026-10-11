//! Compression zstd des sources brutes (niveau 3).

use mira_vocabulary::error::codes::local_storage;
use mira_vocabulary::{ErrorCategory, MiraError};

const LEVEL: i32 = 3;

/// Compresse une source brute
pub(super) fn compress(raw: &[u8]) -> Result<Vec<u8>, MiraError> {
    zstd::stream::encode_all(raw, LEVEL).map_err(|error| io_error(&error))
}

/// Décompresse une source brute lue en base
pub(super) fn decompress(compressed: &[u8]) -> Result<Vec<u8>, MiraError> {
    zstd::stream::decode_all(compressed).map_err(|error| io_error(&error))
}

/// Erreur de compression ou de décompression
fn io_error(error: &std::io::Error) -> MiraError {
    MiraError::new(
        ErrorCategory::LocalStorage,
        local_storage::IO,
        format!("compression de la source brute : {error}"),
    )
}
