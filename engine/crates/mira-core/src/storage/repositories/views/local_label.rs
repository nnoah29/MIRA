//! Liste des messages dont le fil porte un libellé local, sans le masquage de la Réception.

use mira_vocabulary::{AccountId, LocalLabelId};

use super::builder::QueryBuilder;

/// Requête des messages visibles des fils portant ce libellé, sur les comptes donnés
pub(super) fn local_label_builder(accounts: &[AccountId], label: &LocalLabelId) -> QueryBuilder {
    let mut builder = QueryBuilder::new("messages m");
    let names: Vec<&str> = accounts.iter().map(AccountId::as_str).collect();
    let list = builder.bind_list(&names);
    let bound = builder.bind(label.as_str().to_string());
    builder.require(format!("+m.account_id IN {list}"));
    builder.require(format!(
        "EXISTS (SELECT 1 FROM threads ht JOIN thread_local_labels hl ON hl.account_id = ht.account_id AND hl.root_key = ht.root_key WHERE ht.id = m.thread_id AND hl.label_id = {bound})"
    ));
    builder.require(
        "EXISTS (SELECT 1 FROM appearances a WHERE a.message_id = m.id AND a.deleted = 0)",
    );
    builder
}
