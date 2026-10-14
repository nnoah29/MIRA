//! Contrôle du paramètre `state` d'une redirection, en temps constant.

use mira_vocabulary::MiraError;
use mira_vocabulary::error::codes::authentication;

/// Indique si le `state` reçu est celui attendu, sans s'arrêter au premier octet différent
pub(super) fn state_matches(given: Option<&str>, expected: &str) -> bool {
    let Some(given) = given else {
        return false;
    };
    let mut difference = given.len() ^ expected.len();
    for (index, byte) in given.bytes().enumerate() {
        difference |= usize::from(byte ^ expected.as_bytes().get(index).copied().unwrap_or(0));
    }
    difference == 0
}

/// Indique si l'erreur vient d'un `state` inattendu
pub(super) fn is_state_mismatch(error: &MiraError) -> bool {
    error.code == authentication::OAUTH_STATE_MISMATCH.0
}
