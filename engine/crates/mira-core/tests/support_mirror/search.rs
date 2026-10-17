//! Aides de test pour la recherche plein texte.

use mira_core::storage::Database;
use mira_vocabulary::{AccountId, MessageId, PageRequest};

/// Identifiants des résultats d'une recherche, du plus récent au plus ancien
pub fn found(db: &Database, account: Option<&str>, query: &str) -> Vec<String> {
    let scope = account.map(AccountId::from_raw);
    let page = db
        .read(|tx| tx.search_messages(scope.as_ref(), query, &PageRequest::default()))
        .unwrap();
    page.items
        .iter()
        .map(|m| m.id.as_str().to_string())
        .collect()
}

/// Indexe un message avec un sujet et un corps donnés
pub fn index(db: &Database, id: &str, subject: &str, body: &str) {
    db.write(|tx| tx.index_message(&MessageId::from_raw(id), subject, "bob@example.org", body))
        .unwrap();
}
