//! Demandes de liste et comptes du profil de test.

use mira_core::reading::{ListQuery, SearchFilters, search};
use mira_core::storage::Database;
use mira_vocabulary::{AccountId, PageRequest, StandardView, ViewRef};

/// Les deux comptes du profil de test
pub fn profile() -> Vec<AccountId> {
    vec![AccountId::from_raw("acc-1"), AccountId::from_raw("acc-2")]
}

/// Première page d'une vue standard, sans filtre
pub fn standard(view: StandardView) -> ListQuery {
    ListQuery::of(ViewRef::Standard { view })
}

/// Identifiants trouvés par une recherche sur les deux comptes, première page
pub fn found(db: &Database, text: &str, filters: &SearchFilters) -> Vec<String> {
    let page = search(db, &profile(), text, filters, &PageRequest::default()).unwrap();
    page.items.iter().map(|m| m.id.to_string()).collect()
}
