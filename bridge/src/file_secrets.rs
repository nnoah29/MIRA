//! Trousseau chiffré dans un fichier, pour les systèmes sans service de secrets (Android).
//! La clé AES-256 vient de l'application, qui la garde dans le Keystore du système ; le fichier seul ne révèle rien.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;
use std::sync::Mutex;

use mira_assembly::mira_vocabulary::error::codes::secrets;
use mira_assembly::mira_vocabulary::{ErrorCategory, MiraError, SecretLocation};
use mira_assembly::ports::{SecretStore, SecretValue};
use ring::aead::{AES_256_GCM, Aad, LessSafeKey, NONCE_LEN, Nonce, UnboundKey};
use ring::rand::{SecureRandom, SystemRandom};

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

/// Erreur du trousseau, sans jamais citer de valeur
fn unavailable(message: &'static str) -> MiraError {
    MiraError::new(
        ErrorCategory::Secrets,
        secrets::KEYRING_UNAVAILABLE,
        message,
    )
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

    /// Relit et déchiffre toutes les entrées ; un fichier absent est un trousseau vide
    fn load(&self) -> Result<BTreeMap<String, String>, MiraError> {
        let Ok(mut sealed) = fs::read(&self.path) else {
            return Ok(BTreeMap::new());
        };
        if sealed.len() < NONCE_LEN {
            return Err(unavailable("trousseau illisible"));
        }
        let mut body = sealed.split_off(NONCE_LEN);
        let nonce = Nonce::try_assume_unique_for_key(&sealed)
            .map_err(|_| unavailable("trousseau illisible"))?;
        let plain = self
            .key
            .open_in_place(nonce, Aad::empty(), &mut body)
            .map_err(|_| unavailable("trousseau illisible"))?;
        serde_json::from_slice(plain).map_err(|_| unavailable("trousseau illisible"))
    }

    /// Chiffre et réécrit toutes les entrées d'un coup (fichier temporaire puis renommage)
    fn save(&self, entries: &BTreeMap<String, String>) -> Result<(), MiraError> {
        let mut body = serde_json::to_vec(entries).map_err(|_| unavailable("trousseau"))?;
        let mut nonce = [0u8; NONCE_LEN];
        SystemRandom::new()
            .fill(&mut nonce)
            .map_err(|_| unavailable("aléa indisponible"))?;
        self.key
            .seal_in_place_append_tag(Nonce::assume_unique_for_key(nonce), Aad::empty(), &mut body)
            .map_err(|_| unavailable("chiffrement impossible"))?;
        let tmp = self.path.with_extension("tmp");
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&tmp)
            .map_err(|_| unavailable("écriture du trousseau impossible"))?;
        file.write_all(&nonce)
            .and_then(|()| file.write_all(&body))
            .and_then(|()| file.sync_all())
            .map_err(|_| unavailable("écriture du trousseau impossible"))?;
        fs::rename(&tmp, &self.path).map_err(|_| unavailable("écriture du trousseau impossible"))
    }

    /// Applique une modification aux entrées sous le verrou, puis réécrit le fichier
    fn update(&self, change: impl FnOnce(&mut BTreeMap<String, String>)) -> Result<(), MiraError> {
        let _guard = self.lock.lock().map_err(|_| unavailable("trousseau"))?;
        let mut entries = self.load()?;
        change(&mut entries);
        self.save(&entries)
    }
}

impl SecretStore for FileSecretStore {
    /// Enregistre ou remplace le secret
    fn put(&self, key: &dyn SecretLocation, value: &SecretValue) -> Result<(), MiraError> {
        let value = value.expose().to_string();
        self.update(|entries| {
            entries.insert(key.storage_key(), value);
        })
    }

    /// Relit le secret, ou rien s'il n'existe pas
    fn get(&self, key: &dyn SecretLocation) -> Result<Option<SecretValue>, MiraError> {
        let _guard = self.lock.lock().map_err(|_| unavailable("trousseau"))?;
        Ok(self
            .load()?
            .remove(&key.storage_key())
            .map(SecretValue::new))
    }

    /// Supprime le secret ; absent, ce n'est pas une erreur
    fn delete(&self, key: &dyn SecretLocation) -> Result<(), MiraError> {
        self.update(|entries| {
            entries.remove(&key.storage_key());
        })
    }
}
