//! `mira search local` : recherche plein texte dans le miroir, filtrée et paginée.

use clap::Args;
use mira_assembly::api::SearchRequest;
use mira_assembly::mira_vocabulary::{MailboxId, MiraError};
use mira_assembly::reading::SearchFilters;

use crate::commands::filter_args::FilterArgs;
use crate::commands::paging::PageArgs;

/// Options de `mira search local`.
#[derive(Debug, Args)]
pub struct LocalArgs {
    /// Mots cherchés (tous doivent être présents).
    #[arg(required = true)]
    pub words: Vec<String>,
    /// Ne cherche que dans cette boîte.
    #[arg(long)]
    pub mailbox: Option<String>,
    /// Ne garde que les messages dont un destinataire contient ce texte.
    #[arg(long)]
    pub to: Option<String>,
    #[command(flatten)]
    pub filters: FilterArgs,
    #[command(flatten)]
    pub page: PageArgs,
}

/// Construit la requête de recherche locale.
pub fn request(args: &LocalArgs) -> Result<SearchRequest, MiraError> {
    Ok(SearchRequest::Search {
        query: args.words.join(" "),
        filters: SearchFilters {
            common: args.filters.message_filters()?,
            to_contains: args.to.clone(),
            mailbox: args.mailbox.as_deref().map(MailboxId::from_raw),
        },
        page: args.page.page_request(),
    })
}
