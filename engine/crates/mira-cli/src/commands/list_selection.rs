//! Sélection d'une liste de messages : une vue et des filtres, sans pagination.

use clap::Args;
use mira_assembly::mira_vocabulary::{MiraError, PageRequest};
use mira_assembly::reading::ListQuery;

use super::filter_args::FilterArgs;
use super::view_args::ViewArgs;

/// Vue et filtres d'une liste de messages.
#[derive(Debug, Args)]
pub struct ListSelectionArgs {
    #[command(flatten)]
    pub view: ViewArgs,
    #[command(flatten)]
    pub filters: FilterArgs,
}

impl ListSelectionArgs {
    /// Construit la demande de liste du contrat avec la page donnée.
    pub fn list_query(&self, page: PageRequest) -> Result<ListQuery, MiraError> {
        Ok(ListQuery {
            view: self.view.view_ref()?,
            filters: self.filters.message_filters()?,
            page,
        })
    }
}
