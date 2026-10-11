//! Analyse des réponses du serveur : jetons, LIST, FETCH, SELECT, SEARCH, VANISHED, dates.

pub(crate) mod atoms;
pub(crate) mod capability;
pub(crate) mod civil;
pub(crate) mod dates;
pub(crate) mod fetch;
pub(crate) mod fetched;
pub(crate) mod idle;
pub(crate) mod list;
pub(crate) mod render;
pub(crate) mod search;
pub(crate) mod select;
pub(crate) mod status;
pub(crate) mod token;
pub(crate) mod tokenizer;
pub(crate) mod uid_set;
pub(crate) mod vanished;
