//! Chargement du fichier de configuration OAuth : absent = vide, illisible = erreur.

use std::io::ErrorKind;
use std::path::Path;

use mira_core::auth::OAuthClientConfig;
use mira_vocabulary::error::codes::local_storage;
use mira_vocabulary::{ErrorCategory, MiraError};

use crate::oauth_embedded::embedded_oauth_config;
use crate::oauth_file_parse::parse_oauth_config;
use crate::oauth_file_path::oauth_config_path_from_process;
use crate::oauth_overlay::overlay;

/// Lit le fichier donné ; un fichier absent donne une configuration vide
pub fn load_oauth_config(path: &Path) -> Result<OAuthClientConfig, MiraError> {
    match std::fs::read_to_string(path) {
        Ok(text) => parse_oauth_config(&text),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(OAuthClientConfig::default()),
        Err(error) => Err(MiraError::new(
            ErrorCategory::LocalStorage,
            local_storage::IO,
            format!("lecture de {} impossible : {error}", path.display()),
        )),
    }
}

/// Configuration OAuth du processus : identifiants intégrés, recouverts par le fichier ; un fichier invalide est signalé puis ignoré
pub fn oauth_config_from_process() -> OAuthClientConfig {
    let embedded = embedded_oauth_config();
    let Some(path) = oauth_config_path_from_process() else {
        return embedded;
    };
    match load_oauth_config(&path) {
        Ok(local) => overlay(embedded, local),
        Err(error) => {
            tracing::warn!(path = %path.display(), %error, "configuration OAuth ignorée");
            embedded
        }
    }
}
