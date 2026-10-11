//! PKCE (RFC 7636, méthode S256) : vérificateur, défi et jetons aléatoires.

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::RngCore;
use sha2::{Digest, Sha256};

use crate::ports::SecretValue;

/// Octets aléatoires du vérificateur : 48 octets donnent 64 caractères non réservés
const VERIFIER_BYTES: usize = 48;

/// Tire `byte_count` octets aléatoires et les écrit en base64url sans remplissage
pub fn random_token(rng: &mut dyn RngCore, byte_count: usize) -> String {
    let mut bytes = vec![0_u8; byte_count];
    rng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Génère un `code_verifier` de 64 caractères non réservés
pub fn generate_verifier(rng: &mut dyn RngCore) -> SecretValue {
    SecretValue::new(random_token(rng, VERIFIER_BYTES))
}

/// Calcule `BASE64URL(SHA256(verifier))` sans remplissage
pub fn challenge_for(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}
