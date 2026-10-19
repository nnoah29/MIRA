//! Identifiants OAuth intégrés au binaire à la compilation (`build.rs`), chiffrés en AES-256-GCM.

use mira_core::auth::OAuthClientConfig;
use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{ErrorCategory, MiraError};
use ring::aead::{AES_256_GCM, Aad, LessSafeKey, Nonce, UnboundKey};

use crate::oauth_file_parse::parse_oauth_config;

mod sealed {
    include!(concat!(env!("OUT_DIR"), "/embedded_oauth.rs"));
}

/// Configuration intégrée au binaire ; vide si la compilation n'en a pas reçu ou si elle est illisible
pub fn embedded_oauth_config() -> OAuthClientConfig {
    if sealed::CIPHERTEXT.is_empty() {
        return OAuthClientConfig::default();
    }
    open_embedded(sealed::KEY, sealed::NONCE, sealed::CIPHERTEXT).unwrap_or_else(|error| {
        tracing::warn!(%error, "identifiants OAuth intégrés ignorés");
        OAuthClientConfig::default()
    })
}

/// Déchiffre puis décode une configuration scellée par `build.rs`
pub fn open_embedded(
    key: &[u8],
    nonce: &[u8],
    ciphertext: &[u8],
) -> Result<OAuthClientConfig, MiraError> {
    let unbound = UnboundKey::new(&AES_256_GCM, key).map_err(|_| unreadable())?;
    let nonce = Nonce::try_assume_unique_for_key(nonce).map_err(|_| unreadable())?;
    let mut buffer = ciphertext.to_vec();
    let plain = LessSafeKey::new(unbound)
        .open_in_place(nonce, Aad::empty(), &mut buffer)
        .map_err(|_| unreadable())?;
    let text = std::str::from_utf8(plain).map_err(|_| unreadable())?;
    parse_oauth_config(text)
}

/// Erreur sans détail : le contenu déchiffré ne doit pas fuir
fn unreadable() -> MiraError {
    MiraError::new(
        ErrorCategory::InvalidInput,
        invalid_input::PARAMETER,
        "identifiants OAuth intégrés illisibles",
    )
}
