//! En-têtes `In-Reply-To` et `References` d'un brouillon, construits depuis le message d'origine.

use mira_vocabulary::{Draft, MessageSummary, MiraError};

use crate::storage::Tx;

/// En-têtes de rattachement à une conversation, tels que l'envoi les écrira.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ThreadingHeaders {
    pub in_reply_to: Option<String>,
    pub references: Vec<String>,
}

/// En-têtes d'une réponse (ou d'un transfert) au message : son `Message-ID`, et ses références suivies de ce `Message-ID`
pub fn threading_headers(original: &MessageSummary) -> ThreadingHeaders {
    let mut references = original.references.clone();
    if let Some(id) = &original.message_id_header
        && !references.contains(id)
    {
        references.push(id.clone());
    }
    ThreadingHeaders {
        in_reply_to: original.message_id_header.clone(),
        references,
    }
}

/// En-têtes du brouillon : vides s'il ne répond à rien ou si le message d'origine a disparu du miroir
pub fn draft_threading_headers(tx: &Tx<'_>, draft: &Draft) -> Result<ThreadingHeaders, MiraError> {
    let Some(original_id) = &draft.in_reply_to else {
        return Ok(ThreadingHeaders::default());
    };
    Ok(tx
        .get_message(original_id)?
        .map(|original| threading_headers(&original))
        .unwrap_or_default())
}
