//! Rejeu de la file d'un compte vers le serveur, dans l'ordre de soumission (inv. 19).

use mira_vocabulary::{MiraError, PendingAction};

use super::attempt::{begin_attempt, complete_attempt};
use super::classify::{FailureClass, classify};
use super::context::ReplayContext;
use super::execute::execute_action;
use super::park::require_reauthentication;
use super::report::{ReplayReport, ReplayStop, StepOutcome};
use super::settle::settle_failure;
use crate::imap::ImapClient;

/// Rejoue les actions prêtes, une à une ; s'arrête au premier échec passagère ou d'authentification
pub async fn replay_actions(
    ctx: &ReplayContext<'_>,
    client: &mut ImapClient,
) -> Result<ReplayReport, MiraError> {
    let mut report = ReplayReport::default();
    if next_ready(ctx)?.is_none() {
        return Ok(report);
    }
    if let Some(stop) = renew_credentials(ctx).await? {
        report.stopped = Some(stop);
        return Ok(report);
    }
    while let Some(action) = next_ready(ctx)? {
        let outcome = replay_one(ctx, client, &action).await?;
        if report.record(outcome) {
            break;
        }
    }
    Ok(report)
}

/// Première action du compte prête à partir, dans l'ordre de la file
fn next_ready(ctx: &ReplayContext<'_>) -> Result<Option<PendingAction>, MiraError> {
    let now = ctx.queue.clock.now();
    ctx.queue
        .db
        .read(|tx| tx.next_ready_action(ctx.account, now))
}

/// Une tentative complète : « en cours » écrit, serveur, puis succès ou règle d'échec
async fn replay_one(
    ctx: &ReplayContext<'_>,
    client: &mut ImapClient,
    action: &PendingAction,
) -> Result<StepOutcome, MiraError> {
    if !begin_attempt(ctx, action)? {
        return Ok(StepOutcome::Skipped);
    }
    match execute_action(ctx, client, action).await {
        Ok(()) => complete_attempt(ctx, action),
        Err(error) => settle_failure(ctx, client, action, &error).await,
    }
}

/// Rafraîchit en silence le jeton expiré (S10) ; si c'est impossible, le compte demande une authentification
async fn renew_credentials(ctx: &ReplayContext<'_>) -> Result<Option<ReplayStop>, MiraError> {
    let Err(error) = ctx.credentials.renew_if_expired(ctx.account).await else {
        return Ok(None);
    };
    if classify(&error) == FailureClass::Authentication {
        require_reauthentication(ctx)?;
        return Ok(Some(ReplayStop::AuthRequired));
    }
    Ok(Some(ReplayStop::Retry { code: error.code }))
}
