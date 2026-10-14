//! `Message-ID` d'un envoi : généré par le moteur, mémorisé dans l'action avant l'appel SMTP (10.8, inv. 26).

use mira_vocabulary::{ActionId, DraftId, DraftState, MiraError};

use crate::actions::errors::not_found::action_not_found;
use crate::actions::load_server_targets;
use crate::drafts::state::set_draft_state;
use crate::ports::IdGenerator;
use crate::storage::Tx;
use crate::storage::json_columns::to_json;
use crate::storage::repositories::actions::ActionPayload;

/// Prépare une tentative d'envoi : `Message-ID` fixé une fois pour toutes, brouillon figé en « envoi en cours »
pub(crate) fn prepare_send_attempt(
    tx: &Tx<'_>,
    ids: &dyn IdGenerator,
    action: &ActionId,
    draft: &DraftId,
) -> Result<(), MiraError> {
    ensure_message_id(tx, ids, action)?;
    if tx.get_draft(draft)?.is_some() {
        set_draft_state(tx, draft, DraftState::Sending)?;
    }
    Ok(())
}

/// Rend le `Message-ID` mémorisé de l'action, en le générant à la première demande
pub(crate) fn ensure_message_id(
    tx: &Tx<'_>,
    ids: &dyn IdGenerator,
    action: &ActionId,
) -> Result<String, MiraError> {
    let mut targets = load_server_targets(tx, action)?;
    if let Some(existing) = &targets.sent_message_id {
        return Ok(existing.clone());
    }
    let fresh = format!("<{}@mira.local>", ids.next_id());
    targets.sent_message_id = Some(fresh.clone());
    save_targets(tx, action, &targets)?;
    Ok(fresh)
}

/// `Message-ID` mémorisé de l'envoi, s'il a déjà été généré
pub(crate) fn stored_message_id(
    tx: &Tx<'_>,
    action: &ActionId,
) -> Result<Option<String>, MiraError> {
    Ok(load_server_targets(tx, action)?.sent_message_id)
}

/// Oublie le `Message-ID` : une nouvelle tentative voulue par l'utilisateur en aura un neuf
pub(crate) fn clear_message_id(tx: &Tx<'_>, action: &ActionId) -> Result<(), MiraError> {
    let mut targets = load_server_targets(tx, action)?;
    targets.sent_message_id = None;
    save_targets(tx, action, &targets)
}

/// Réécrit les cibles serveur de l'action en gardant son enregistrement d'annulation
fn save_targets(
    tx: &Tx<'_>,
    action: &ActionId,
    targets: &crate::actions::ServerTargets,
) -> Result<(), MiraError> {
    let payload = tx
        .get_action_payload(action)?
        .ok_or_else(|| action_not_found(action))?;
    tx.set_action_payload(
        action,
        &ActionPayload {
            server_targets_json: to_json(targets)?,
            undo_json: payload.undo_json,
        },
    )
}
