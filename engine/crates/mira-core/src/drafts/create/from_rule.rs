//! Brouillon de réponse créé par une règle, une seule fois par message et par rang d'action.

use mira_vocabulary::{Draft, Identity, MessageId, MessageSummary, MiraError};

use super::fields::{Mode, fields_from};
use super::insert::{insert_new_draft, own_addresses};
use crate::actions::QueueContext;
use crate::drafts::identity::resolve_identity;
use crate::storage::repositories::messages::message_not_found;

/// Objet (facultatif) et corps rendus par le modèle d'une règle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleReplyText {
    pub subject: Option<String>,
    pub body: String,
}

/// Crée la réponse au message avec le texte rendu ; rien si ce rang a déjà donné un brouillon
pub fn create_rule_reply(
    ctx: &QueueContext<'_>,
    original: &MessageId,
    rank: usize,
    render: impl FnOnce(&Identity, &MessageSummary) -> RuleReplyText,
) -> Result<Option<Draft>, MiraError> {
    ctx.db.write(|tx| {
        if tx.rule_reply_draft(original, rank)?.is_some() {
            return Ok(None);
        }
        let message = tx
            .get_message(original)?
            .ok_or_else(|| message_not_found(original))?;
        let identity = resolve_identity(tx, &message.account, None)?;
        let own = own_addresses(tx, &message.account)?;
        let fields = fields_from(Mode::Reply, &message, &own);
        let mut draft = insert_new_draft(tx, ctx, &message.account, &identity, fields)?;
        let text = render(&identity, &message);
        draft.body_text = text.body;
        if let Some(subject) = text.subject.filter(|subject| !subject.is_empty()) {
            draft.subject = subject;
        }
        tx.update_draft(&draft)?;
        tx.record_rule_reply_draft(original, rank, &draft.id)?;
        Ok(Some(draft))
    })
}
