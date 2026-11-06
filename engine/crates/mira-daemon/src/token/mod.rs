//! Jeton local du démon : 256 bits aléatoires dans un fichier privé, comparé en temps constant.

mod file;

use std::path::Path;

use ring::hmac::{self, HMAC_SHA256};

/// Secret que tout client local présente dans `hello` ; jamais journalisé ni renvoyé.
pub struct Token(String);

impl Token {
    /// Relit le jeton du fichier, ou le crée (dossier parent compris) s'il n'existe pas.
    pub fn load_or_create(path: &Path) -> std::io::Result<Self> {
        match file::read_existing(path)? {
            Some(text) => Ok(Self(text)),
            None => file::create_private(path).map(Self),
        }
    }

    /// Dit si `given` est exactement ce jeton ; la comparaison des empreintes HMAC est en temps constant.
    pub fn matches(&self, given: &str) -> bool {
        let key = hmac::Key::new(HMAC_SHA256, b"mira-daemon-token");
        let expected = hmac::sign(&key, self.0.as_bytes());
        hmac::verify(&key, given.as_bytes(), expected.as_ref()).is_ok()
    }

    /// Le jeton en hexadécimal, pour l'écrire à un client de confiance.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for Token {
    /// N'affiche jamais le jeton
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Token(..)")
    }
}
