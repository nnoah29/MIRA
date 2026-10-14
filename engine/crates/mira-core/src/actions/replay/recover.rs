//! Reprise au démarrage : les actions laissées « en cours » par un arrêt brutal sont réglées sans renvoi (inv. 18, 26).

use mira_vocabulary::{ActionKind, ActionState, MiraError, PendingAction};

use super::context::ReplayContext;
use super::report::StepOutcome;
use crate::imap::ImapClient;
use crate::send::recover_interrupted_send;

/// Ce que la reprise a décidé pour les actions interrompues.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecoveryReport {
    pub requeued: u32,
    pub sends_confirmed: u32,
    pub sends_uncertain: u32,
}

/// Décision prise pour une action interrompue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Recovered {
    Requeued,
    Confirmed,
    Uncertain,
}

/// Règle chaque action « en cours » du compte : un envoi est vérifié dans Envoyés (jamais renvoyé), le reste est rejoué
pub async fn recover_interrupted_actions(
    ctx: &ReplayContext<'_>,
    client: &mut ImapClient,
) -> Result<RecoveryReport, MiraError> {
    let interrupted = ctx
        .queue
        .db
        .read(|tx| tx.list_actions(ctx.account, &[ActionState::InProgress]))?;
    let mut report = RecoveryReport::default();
    for action in interrupted {
        match recover_one(ctx, client, &action).await? {
            Recovered::Requeued => report.requeued += 1,
            Recovered::Confirmed => report.sends_confirmed += 1,
            Recovered::Uncertain => report.sends_uncertain += 1,
        }
    }
    Ok(report)
}

/// Règle une action interrompue : un envoi est vérifié, toute autre action est remise en file
async fn recover_one(
    ctx: &ReplayContext<'_>,
    client: &mut ImapClient,
    action: &PendingAction,
) -> Result<Recovered, MiraError> {
    if matches!(action.kind, ActionKind::Send { .. }) {
        return match recover_interrupted_send(ctx, client, action).await? {
            StepOutcome::Succeeded => Ok(Recovered::Confirmed),
            StepOutcome::Uncertain
            | StepOutcome::Failed
            | StepOutcome::Skipped
            | StepOutcome::Stop(_) => Ok(Recovered::Uncertain),
        };
    }
    ctx.queue
        .db
        .write(|tx| tx.set_action_state(&action.id, ActionState::Pending))?;
    Ok(Recovered::Requeued)
}
