//! Demande de liste : une vue, des filtres et une page.

use mira_vocabulary::{PageRequest, ViewRef};
use serde::{Deserialize, Serialize};

pub use crate::storage::repositories::views::{MessageFilters, SearchFilters};

/// Ce que le client veut lister : la vue, les filtres et la page demandée.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListQuery {
    pub view: ViewRef,
    pub filters: MessageFilters,
    pub page: PageRequest,
}

impl ListQuery {
    /// Demande la première page d'une vue, sans filtre
    pub fn of(view: ViewRef) -> Self {
        Self {
            view,
            filters: MessageFilters::default(),
            page: PageRequest::default(),
        }
    }
}
