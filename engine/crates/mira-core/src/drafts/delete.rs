//! Suppression d'un brouillon, avec celle de sa copie sur le serveur quand elle existe.

use mira_vocabulary::{
    ActionKind, ActionState, Draft, DraftId, DraftState, EventKind, IdempotencyKey, MiraError,
    ProviderQuirks,
};

use crate::actions::errors::incompatible::draft_not_editable;
use crate::actions::errors::not_found::draft_not_found;
use crate::actions::{QueueContext, Queued, SubmitRequest, UNDO_DELAY_MILLIS, queue_action};
use crate::storage::Tx;

/// Vrai si le serveur a reçu (ou reçoit) une copie du brouillon par une action `SaveDraft`
fn has_server_copy(tx: &Tx<'_>, draft: &Draft) -> Result<bool, MiraError> {
    let states = [ActionState::Succeeded, ActionState::InProgress];
    let actions = tx.list_actions(&draft.account, &states)?;
    Ok(actions.iter().any(
        |action| matches!(&action.kind, ActionKind::SaveDraft { draft: id } if *id == draft.id),
    ))
}

/// Met en file le rejeu qui retirera du serveur la copie d'un brouillon qui va disparaître
fn queue_server_copy_removal(
    tx: &Tx<'_>,
    ctx: &QueueContext<'_>,
    draft: &Draft,
) -> Result<Queued, MiraError> {
    let request = SubmitRequest {
        account: draft.account.clone(),
        kind: ActionKind::SaveDraft {
            draft: draft.id.clone(),
        },
        targets: Vec::new(),
        key: IdempotencyKey(format!("remove-draft-copy-{}", draft.id)),
    };
    let when = ctx.clock.now().plus_millis(UNDO_DELAY_MILLIS);
    queue_action(tx, ctx, &ProviderQuirks::default(), &request, when)
}

/// Supprime un brouillon en édition avec ses pièces jointes ; annulez d'abord l'envoi d'un brouillon planifié
pub fn delete_draft(ctx: &QueueContext<'_>, id: &DraftId) -> Result<(), MiraError> {
    let queued = ctx.db.write(|tx| {
        let draft = tx.get_draft(id)?.ok_or_else(|| draft_not_found(id))?;
        if draft.state != DraftState::Editing {
            return Err(draft_not_editable());
        }
        let queued = if has_server_copy(tx, &draft)? {
            Some(queue_server_copy_removal(tx, ctx, &draft)?)
        } else {
            None
        };
        tx.delete_draft(id)?;
        Ok(queued)
    })?;
    if let Some(queued) = queued.filter(|queued| queued.is_new) {
        ctx.events.emit(EventKind::ActionQueued {
            account: queued.action.account,
            action: queued.action.id,
        })?;
    }
    Ok(())
}
