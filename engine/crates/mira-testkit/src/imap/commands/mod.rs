//! Gestionnaires de commandes IMAP, un module par commande.

pub(crate) mod append;
pub(crate) mod append_parts;
pub(crate) mod args;
pub(crate) mod authenticate;
pub(crate) mod capability;
pub(crate) mod copy;
pub(crate) mod enable;
pub(crate) mod expunge;
pub(crate) mod fetch;
pub(crate) mod id;
pub(crate) mod idle;
pub(crate) mod kind;
pub(crate) mod list;
pub(crate) mod login;
pub(crate) mod manage;
pub(crate) mod move_cmd;
pub(crate) mod router;
pub(crate) mod search;
pub(crate) mod select;
pub(crate) mod status;
pub(crate) mod store_cmd;
pub(crate) mod transfer;
