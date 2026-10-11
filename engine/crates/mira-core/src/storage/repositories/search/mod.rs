//! Dépôt « search » : index plein texte FTS5 des messages, requêtes échappées.

mod index;
mod query;

pub(crate) use query::escape_match_query;
