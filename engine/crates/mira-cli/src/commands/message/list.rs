//! `mira message list` : messages d'une vue, filtrés et paginés (lecture locale).

use clap::Args;
use mira_assembly::api::MessageRequest;
use mira_assembly::mira_vocabulary::MiraError;

use crate::commands::list_selection::ListSelectionArgs;
use crate::commands::paging::PageArgs;

/// Options de `mira message list`.
#[derive(Debug, Args)]
pub struct ListArgs {
    #[command(flatten)]
    pub selection: ListSelectionArgs,
    #[command(flatten)]
    pub page: PageArgs,
}

/// Construit la requête de liste de messages.
pub fn request(args: &ListArgs) -> Result<MessageRequest, MiraError> {
    let query = args.selection.list_query(args.page.page_request())?;
    Ok(MessageRequest::ListMessages(query))
}
