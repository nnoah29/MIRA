//! Dépôt « messages » : résumés de messages, drapeaux, état du contenu et listes paginées.

mod content_state;
mod errors;
mod query;
mod read;
mod rows;
mod update;
mod values;
mod write;

pub(crate) use errors::message_not_found;
pub(crate) use rows::COLUMNS as MESSAGE_COLUMNS;
