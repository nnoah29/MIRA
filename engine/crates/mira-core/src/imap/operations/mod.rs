//! Opérations de haut niveau du client IMAP, une par fichier (aucune syntaxe IMAP n'en sort).

mod append;
mod enable;
mod expunge;
mod fetch_flags;
mod fetch_headers;
mod fetch_source;
mod idle;
mod labels;
mod list_mailboxes;
mod logout;
mod mailbox_admin;
mod search;
mod search_utf8;
mod select;
mod store_flags;
mod transfer;
