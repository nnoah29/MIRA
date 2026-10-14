//! Cibles d'une action en lot : une liste de messages ou le résultat d'une requête de liste.

use mira_assembly::api::TargetSelection;
use mira_assembly::mira_vocabulary::{MessageId, MiraError, PageRequest};

use crate::argument_error::invalid_argument;
use crate::commands::list_selection::ListSelectionArgs;

/// Choisit les cibles : `--message` répétés, ou une vue (`--mailbox`/`--view`) et ses filtres.
pub fn target_selection(
    messages: &[String],
    selection: &ListSelectionArgs,
) -> Result<TargetSelection, MiraError> {
    match (messages.is_empty(), selection.view.is_given()) {
        (false, false) => Ok(TargetSelection::Messages {
            messages: messages.iter().map(MessageId::from_raw).collect(),
        }),
        (true, true) => Ok(TargetSelection::Query {
            query: selection.list_query(PageRequest::default())?,
        }),
        (false, true) => Err(invalid_argument(
            "--message est exclusif avec --mailbox et --view",
        )),
        (true, false) => Err(invalid_argument(
            "indiquer des --message, ou --mailbox/--view pour une sélection par requête",
        )),
    }
}
