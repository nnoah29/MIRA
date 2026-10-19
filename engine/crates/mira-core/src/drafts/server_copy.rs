//! Mise en file de la copie d'un brouillon dans la boîte Brouillons du serveur.

use mira_vocabulary::{
    ActionId, ActionKind, DraftId, DraftState, EventKind, IdempotencyKey, MiraError, ProviderQuirks,
};

use crate::actions::errors::incompatible::draft_not_editable;
use crate::actions::errors::not_found::draft_not_found;
use crate::actions::{QueueContext, SubmitRequest, queue_action};

/// Met en file l'enregistrement serveur d'un brouillon en édition ; la même clé rend la même action
pub fn queue_draft_copy(
    ctx: &QueueContext<'_>,
    id: &DraftId,
    key: IdempotencyKey,
) -> Result<ActionId, MiraError> {
    let queued = ctx.db.write(|tx| {
        let draft = tx.get_draft(id)?.ok_or_else(|| draft_not_found(id))?;
        if draft.state != DraftState::Editing {
            return Err(draft_not_editable());
        }
        let request = SubmitRequest {
            account: draft.account,
            kind: ActionKind::SaveDraft { draft: draft.id },
            targets: Vec::new(),
            key,
        };
        let when = ctx.clock.now();
        queue_action(tx, ctx, &ProviderQuirks::default(), &request, when)
    })?;
    if queued.is_new {
        ctx.events.emit(EventKind::ActionQueued {
            account: queued.action.account.clone(),
            action: queued.action.id.clone(),
        })?;
    }
    Ok(queued.action.id)
}
