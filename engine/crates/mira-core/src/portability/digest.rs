//! Empreinte d'un fichier d'import, mémorisée avec sa clé d'idempotence à la place du texte.

use sha2::{Digest, Sha256};

/// Empreinte hexadécimale (SHA-256) du texte
pub fn digest_of(text: &str) -> String {
    Sha256::digest(text.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
