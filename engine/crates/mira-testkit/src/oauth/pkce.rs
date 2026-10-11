//! PKCE avec la méthode S256 (RFC 7636 section 4).

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};

/// Calcule BASE64URL(SHA256(vérificateur)) sans bourrage
pub fn challenge_for(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

/// Vérifie la forme d'un vérificateur : 43 à 128 caractères non réservés
pub fn is_valid_verifier(verifier: &str) -> bool {
    let allowed = |byte: u8| byte.is_ascii_alphanumeric() || b"-._~".contains(&byte);
    (43..=128).contains(&verifier.len()) && verifier.bytes().all(allowed)
}

/// Vérifie qu'un vérificateur bien formé correspond au défi mémorisé
pub fn verifies(verifier: &str, challenge: &str) -> bool {
    is_valid_verifier(verifier) && challenge_for(verifier) == challenge
}
