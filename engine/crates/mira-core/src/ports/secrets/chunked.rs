//! Secrets longs découpés en morceaux : certains trousseaux (Windows : 2560 octets, soit 1280 caractères) refusent les jetons longs.

use mira_vocabulary::{MiraError, SecretLocation};

use super::chunk_parts::{PartKey, count_in, join_parts, marker_for, remove_parts, split};
use super::store::SecretStore;
use super::value::SecretValue;

/// Enveloppe un trousseau : un secret de plus de `max_chars` caractères est écrit en plusieurs entrées, relues d'un bloc.
#[derive(Debug)]
pub struct ChunkedSecretStore<S> {
    inner: S,
    max_chars: usize,
}

impl<S: SecretStore> ChunkedSecretStore<S> {
    /// Enveloppe `inner` ; `max_chars` est la taille maximale d'un morceau.
    pub fn new(inner: S, max_chars: usize) -> Self {
        Self { inner, max_chars }
    }

    /// Le trousseau enveloppé, tel que le système le voit.
    pub fn inner(&self) -> &S {
        &self.inner
    }

    /// Nombre de morceaux que le secret occupe déjà ; zéro s'il est entier ou absent.
    fn parts_in_use(&self, key: &dyn SecretLocation) -> Result<usize, MiraError> {
        let stored = self.inner.get(key)?;
        Ok(stored.and_then(|v| count_in(v.expose())).unwrap_or(0))
    }
}

impl<S: SecretStore> SecretStore for ChunkedSecretStore<S> {
    /// Écrit le secret entier s'il est court, sinon ses morceaux puis l'entrée qui les annonce ; les morceaux en trop sont effacés.
    fn put(&self, key: &dyn SecretLocation, value: &SecretValue) -> Result<(), MiraError> {
        let previous = self.parts_in_use(key)?;
        let parts = split(value.expose(), self.max_chars);
        if parts.len() <= 1 {
            self.inner.put(key, value)?;
            return remove_parts(&self.inner, key, 0..previous);
        }
        for (index, part) in parts.iter().enumerate() {
            let location = PartKey {
                base: key.storage_key(),
                index,
            };
            self.inner
                .put(&location, &SecretValue::new((*part).to_string()))?;
        }
        self.inner
            .put(key, &SecretValue::new(marker_for(parts.len())))?;
        remove_parts(&self.inner, key, parts.len()..previous)
    }

    /// Relit le secret entier, ou recolle ses morceaux
    fn get(&self, key: &dyn SecretLocation) -> Result<Option<SecretValue>, MiraError> {
        let Some(stored) = self.inner.get(key)? else {
            return Ok(None);
        };
        match count_in(stored.expose()) {
            Some(count) => join_parts(&self.inner, key, count),
            None => Ok(Some(stored)),
        }
    }

    /// Supprime le secret et tous ses morceaux
    fn delete(&self, key: &dyn SecretLocation) -> Result<(), MiraError> {
        let used = self.parts_in_use(key)?;
        remove_parts(&self.inner, key, 0..used)?;
        self.inner.delete(key)
    }
}
