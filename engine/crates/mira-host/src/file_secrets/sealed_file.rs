//! Lecture et écriture du fichier chiffré : nonce, texte chiffré, renommage atomique.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use mira_assembly::mira_vocabulary::error::codes::secrets;
use mira_assembly::mira_vocabulary::{ErrorCategory, MiraError};
use ring::aead::{Aad, LessSafeKey, NONCE_LEN, Nonce};
use ring::rand::{SecureRandom, SystemRandom};

/// Erreur du trousseau, sans jamais citer de valeur
pub(super) fn unavailable(message: &'static str) -> MiraError {
    MiraError::new(
        ErrorCategory::Secrets,
        secrets::KEYRING_UNAVAILABLE,
        message,
    )
}

/// Relit et déchiffre toutes les entrées ; un fichier absent est un trousseau vide
pub(super) fn load(path: &Path, key: &LessSafeKey) -> Result<BTreeMap<String, String>, MiraError> {
    let Ok(mut sealed) = fs::read(path) else {
        return Ok(BTreeMap::new());
    };
    if sealed.len() < NONCE_LEN {
        return Err(unavailable("trousseau illisible"));
    }
    let mut body = sealed.split_off(NONCE_LEN);
    let nonce = Nonce::try_assume_unique_for_key(&sealed)
        .map_err(|_| unavailable("trousseau illisible"))?;
    let plain = key
        .open_in_place(nonce, Aad::empty(), &mut body)
        .map_err(|_| unavailable("trousseau illisible"))?;
    serde_json::from_slice(plain).map_err(|_| unavailable("trousseau illisible"))
}

/// Chiffre et réécrit toutes les entrées d'un coup (fichier temporaire puis renommage)
pub(super) fn save(
    path: &Path,
    key: &LessSafeKey,
    entries: &BTreeMap<String, String>,
) -> Result<(), MiraError> {
    let mut body = serde_json::to_vec(entries).map_err(|_| unavailable("trousseau"))?;
    let mut nonce = [0u8; NONCE_LEN];
    SystemRandom::new()
        .fill(&mut nonce)
        .map_err(|_| unavailable("aléa indisponible"))?;
    key.seal_in_place_append_tag(Nonce::assume_unique_for_key(nonce), Aad::empty(), &mut body)
        .map_err(|_| unavailable("chiffrement impossible"))?;
    let tmp = path.with_extension("tmp");
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options
        .open(&tmp)
        .map_err(|_| unavailable("écriture du trousseau impossible"))?;
    file.write_all(&nonce)
        .and_then(|()| file.write_all(&body))
        .and_then(|()| file.sync_all())
        .map_err(|_| unavailable("écriture du trousseau impossible"))?;
    fs::rename(&tmp, path).map_err(|_| unavailable("écriture du trousseau impossible"))
}
