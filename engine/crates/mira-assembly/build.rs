//! Chiffre à la compilation la section `[oauth]` du fichier désigné par `MIRA_OAUTH_CONFIG` (AES-256-GCM).
//! La clé, tirée au hasard à chaque compilation, voyage avec le binaire : c'est un masquage, pas un secret.

use std::error::Error;
use std::path::{Path, PathBuf};

use ring::aead::{AES_256_GCM, Aad, LessSafeKey, NONCE_LEN, Nonce, UnboundKey};
use ring::rand::{SecureRandom, SystemRandom};

struct Sealed {
    key: [u8; 32],
    nonce: [u8; NONCE_LEN],
    ciphertext: Vec<u8>,
}

/// Écrit `embedded_oauth.rs` dans OUT_DIR : clé, nonce et texte chiffré, ou tableaux vides sans configuration
fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo::rerun-if-env-changed=MIRA_OAUTH_CONFIG");
    let out = PathBuf::from(std::env::var("OUT_DIR")?).join("embedded_oauth.rs");
    let sealed = match std::env::var_os("MIRA_OAUTH_CONFIG") {
        Some(path) => Some(seal(&oauth_section(Path::new(&path))?)?),
        None => None,
    };
    std::fs::write(out, render(sealed.as_ref()))?;
    Ok(())
}

/// Ne garde du fichier que la table `oauth`, réécrite en TOML
fn oauth_section(path: &Path) -> Result<String, Box<dyn Error>> {
    println!("cargo::rerun-if-changed={}", path.display());
    let file: toml::Table = std::fs::read_to_string(path)?.parse()?;
    let mut kept = toml::Table::new();
    if let Some(oauth) = file.get("oauth") {
        kept.insert("oauth".to_string(), oauth.clone());
    }
    Ok(toml::to_string(&kept)?)
}

/// Chiffre le texte avec une clé et un nonce neufs
fn seal(text: &str) -> Result<Sealed, Box<dyn Error>> {
    let random = SystemRandom::new();
    let mut key = [0u8; 32];
    let mut nonce = [0u8; NONCE_LEN];
    random
        .fill(&mut key)
        .map_err(|_| "tirage de la clé impossible")?;
    random
        .fill(&mut nonce)
        .map_err(|_| "tirage du nonce impossible")?;
    let unbound = UnboundKey::new(&AES_256_GCM, &key).map_err(|_| "clé AES refusée")?;
    let mut ciphertext = text.as_bytes().to_vec();
    LessSafeKey::new(unbound)
        .seal_in_place_append_tag(
            Nonce::assume_unique_for_key(nonce),
            Aad::empty(),
            &mut ciphertext,
        )
        .map_err(|_| "chiffrement impossible")?;
    Ok(Sealed {
        key,
        nonce,
        ciphertext,
    })
}

/// Code Rust des trois constantes lues par `oauth_embedded`
fn render(sealed: Option<&Sealed>) -> String {
    let (key, nonce, ciphertext): (&[u8], &[u8], &[u8]) = match sealed {
        Some(sealed) => (&sealed.key, &sealed.nonce, &sealed.ciphertext),
        None => (&[], &[], &[]),
    };
    format!(
        "pub const KEY: &[u8] = &{key:?};\npub const NONCE: &[u8] = &{nonce:?};\npub const CIPHERTEXT: &[u8] = &{ciphertext:?};\n"
    )
}
