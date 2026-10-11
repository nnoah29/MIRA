//! Analyse syntaxique des commandes IMAP : jetons, ensembles, éléments FETCH et critères SEARCH.

pub(crate) mod atoms;
pub(crate) mod command;
pub(crate) mod error;
pub(crate) mod fetch_items;
pub(crate) mod fetch_section;
pub(crate) mod literal;
pub(crate) mod scanner;
pub(crate) mod search;
pub(crate) mod search_args;
pub(crate) mod search_flag;
pub(crate) mod search_keyword;
pub(crate) mod token;
pub(crate) mod tokenizer;
pub(crate) mod uidset;
pub(crate) mod uidset_format;
