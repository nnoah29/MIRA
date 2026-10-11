//! Dépôt « mailboxes » : boîtes d'un compte, repère de synchronisation et compteurs.

mod read;
mod role;
mod rows;
mod write;

pub(crate) use rows::mailbox_not_found;
