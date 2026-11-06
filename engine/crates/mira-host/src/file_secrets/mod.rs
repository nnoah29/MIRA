//! Trousseau chiffré dans un fichier, pour les systèmes sans service de secrets (Android).
//! La clé AES-256 vient de l'application, qui la garde dans le Keystore du système ; le fichier seul ne révèle rien.

mod sealed_file;
mod store;

use std::path::PathBuf;
use std::sync::Mutex;

use mira_assembly::mira_vocabulary::MiraError;
use ring::aead::{AES_256_GCM, LessSafeKey, UnboundKey};

use self::sealed_file::unavailable;

/// Secrets chiffrés (AES-256-GCM) dans un seul fichier ; chaque écriture tire un nouveau nonce.
pub struct FileSecretStore {
    path: PathBuf,
    key: LessSafeKey,
    lock: Mutex<()>,
}

impl std::fmt::Debug for FileSecretStore {
    /// N'affiche que l'emplacement, jamais la clé
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileSecretStore")
            .field("path", &self.path)
            .finish()
    }
}

impl FileSecretStore {
    /// Prépare le trousseau sur ce fichier avec cette clé de 32 octets
    pub fn new(path: PathBuf, key: &[u8]) -> Result<Self, MiraError> {
        let unbound = UnboundKey::new(&AES_256_GCM, key)
            .map_err(|_| unavailable("clé du trousseau invalide"))?;
        Ok(Self {
            path,
            key: LessSafeKey::new(unbound),
            lock: Mutex::new(()),
        })
    }
}
