//! Inspection du trousseau en mémoire par les tests.

use super::memory::MemorySecretStore;

impl MemorySecretStore {
    /// Nombre de secrets rangés, pour vérifier qu'un échec n'en laisse aucun
    pub fn entry_count(&self) -> usize {
        self.open().map_or(0, |entries| entries.len())
    }
}
