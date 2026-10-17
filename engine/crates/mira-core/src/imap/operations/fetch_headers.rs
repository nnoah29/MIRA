//! Récupération des en-têtes bruts, drapeaux, taille et date interne (étape 2 de la descente).

use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{ErrorCategory, MiraError};

use crate::imap::client::ImapClient;
use crate::imap::errors::status::Operation;
use crate::imap::parse::fetch::parse_fetch;
use crate::imap::parse::fetched::fetched_headers;
use crate::imap::protocol::quoting::uid_sets;
use crate::imap::types::FetchedHeaders;

const BASE_ITEMS: &str = "UID FLAGS RFC822.SIZE INTERNALDATE BODY.PEEK[HEADER]";

/// Vérifie qu'un nom d'attribut supplémentaire ne contient que des caractères sûrs.
fn check_extra(name: &str) -> Result<(), MiraError> {
    let safe = |c: char| c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '_');
    if !name.is_empty() && name.chars().all(safe) {
        return Ok(());
    }
    Err(MiraError::new(
        ErrorCategory::InvalidInput,
        invalid_input::PARAMETER,
        "nom d'attribut supplémentaire invalide",
    ))
}

/// Liste des éléments demandés : base, numéro de modification si possible, attributs du service.
fn header_items(extra: &[String], condstore: bool) -> Result<String, MiraError> {
    let mut items = vec![BASE_ITEMS.to_string()];
    if condstore {
        items.push("MODSEQ".to_string());
    }
    for name in extra {
        check_extra(name)?;
        items.push(name.to_ascii_uppercase());
    }
    Ok(items.join(" "))
}

impl ImapClient {
    /// Récupère les en-têtes des messages (sans les marquer lus) avec les attributs supplémentaires demandés.
    pub async fn fetch_headers(
        &mut self,
        uids: &[u32],
        extra: &[String],
    ) -> Result<Vec<FetchedHeaders>, MiraError> {
        self.require_selected()?;
        let items = header_items(extra, self.has_capability("CONDSTORE"))?;
        let operation = if extra.is_empty() {
            Operation::Plain
        } else {
            Operation::Extension
        };
        let mut found = Vec::new();
        for set in uid_sets(uids) {
            let response = self
                .run(&format!("UID FETCH {set} ({items})"), operation)
                .await?;
            let lines = response.untagged.iter().filter_map(|l| parse_fetch(l));
            found.extend(lines.filter_map(|data| fetched_headers(&data, extra)));
        }
        Ok(found)
    }
}
