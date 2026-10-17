//! Lecture de la réponse CAPABILITY du serveur.

use std::collections::BTreeSet;

use super::server_capability::ServerCapability;

/// Convertit les jetons annoncés en capacités connues ; les jetons inconnus sont ignorés.
pub fn parse_capabilities(raw: &[String]) -> BTreeSet<ServerCapability> {
    raw.iter()
        .filter_map(|token| ServerCapability::from_imap_name(token))
        .collect()
}
