//! Morceaux d'un secret long : clés dérivées, découpe, relecture et nettoyage.

use mira_vocabulary::error::codes::secrets;
use mira_vocabulary::{ErrorCategory, MiraError, SecretLocation};

use super::store::SecretStore;
use super::value::SecretValue;

const MARKER: &str = "\u{1}mira-chunks:";

/// Clé du morceau `index` d'un secret : la clé du secret suivie de `#index`.
pub(super) struct PartKey {
    pub(super) base: String,
    pub(super) index: usize,
}

impl SecretLocation for PartKey {
    /// `<clé>#<numéro>`
    fn storage_key(&self) -> String {
        format!("{}#{}", self.base, self.index)
    }
}

/// Découpe en morceaux d'au plus `max_chars` caractères, sans jamais couper un caractère.
pub(super) fn split(value: &str, max_chars: usize) -> Vec<&str> {
    let starts: Vec<usize> = value
        .char_indices()
        .step_by(max_chars.max(1))
        .map(|(offset, _)| offset)
        .collect();
    let ends = starts.iter().skip(1).copied().chain([value.len()]);
    starts
        .iter()
        .zip(ends)
        .map(|(from, to)| &value[*from..to])
        .collect()
}

/// Valeur stockée à la place du secret quand il est découpé en `count` morceaux.
pub(super) fn marker_for(count: usize) -> String {
    format!("{MARKER}{count}")
}

/// Nombre de morceaux annoncé par une valeur stockée ; rien si c'est un secret ordinaire.
pub(super) fn count_in(stored: &str) -> Option<usize> {
    stored.strip_prefix(MARKER)?.parse().ok()
}

/// Supprime les morceaux `from..to` d'un secret.
pub(super) fn remove_parts(
    inner: &dyn SecretStore,
    key: &dyn SecretLocation,
    range: std::ops::Range<usize>,
) -> Result<(), MiraError> {
    for index in range {
        let part = PartKey {
            base: key.storage_key(),
            index,
        };
        inner.delete(&part)?;
    }
    Ok(())
}

/// Recolle les `count` morceaux ; un morceau absent est une erreur, jamais un secret tronqué.
pub(super) fn join_parts(
    inner: &dyn SecretStore,
    key: &dyn SecretLocation,
    count: usize,
) -> Result<Option<SecretValue>, MiraError> {
    let mut joined = String::new();
    for index in 0..count {
        let part = PartKey {
            base: key.storage_key(),
            index,
        };
        let Some(piece) = inner.get(&part)? else {
            return Err(MiraError::new(
                ErrorCategory::Secrets,
                secrets::KEYRING_UNAVAILABLE,
                "secret incomplet dans le trousseau",
            ));
        };
        joined.push_str(piece.expose());
    }
    Ok(Some(SecretValue::new(joined)))
}
