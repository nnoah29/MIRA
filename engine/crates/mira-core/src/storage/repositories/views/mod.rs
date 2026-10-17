//! Dépôt « views » : listes calculées (boîte, vues standard) et recherche filtrée, paginées par curseur.

mod builder;
mod clauses;
mod filters;
mod list;
mod mailbox;
mod page;
mod search;

pub use filters::{MessageFilters, SearchFilters};
