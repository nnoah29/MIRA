//! Session connectée d'un compte : connexion, descente complète, puis écoute temps réel ou relevés.

use mira_vocabulary::error::codes::network;
use mira_vocabulary::{AccountState, ErrorCategory, MiraError};

use super::exit::{Exit, classify};
use super::idle_start::start_idle;
use super::runtime::AccountRuntime;
use super::states::transition;
use super::wake::{Wake, wait_for_wake};
use crate::sync::download::{sync_account, sync_inbox, sync_requested};
use crate::sync::realtime::{Backoff, IdleWatcher};
use crate::sync::session::{SyncSession, open_connection};

/// Ouvre la connexion de travail dans le budget du compte.
async fn connect(runtime: &AccountRuntime) -> Result<SyncSession, MiraError> {
    let permit = runtime.budget.acquire().await?;
    let credentials = runtime.credentials.as_ref();
    let connection = open_connection(&runtime.ctx, permit, credentials).await?;
    let session = SyncSession::new(runtime.ctx.clone(), connection);
    Ok(session.with_report(runtime.report.clone()))
}

/// Erreur réseau : l'écoute IDLE a perdu sa connexion.
fn idle_lost() -> MiraError {
    MiraError::new(
        ErrorCategory::Network,
        network::CONNECTION_LOST,
        "l'écoute IDLE a perdu sa connexion",
    )
}

/// Sert les demandes en attente en une passe et répond à chaque demandeur ; sans demande, ne fait rien.
async fn serve_requests(
    runtime: &AccountRuntime,
    session: &mut SyncSession,
) -> Result<(), MiraError> {
    let taken = runtime.requests.take();
    if taken.is_empty() {
        return Ok(());
    }
    let outcome = sync_requested(session, &taken).await;
    taken.settle(&outcome);
    outcome
}

/// Attend un réveil puis synchronise (réception seule sur signal IDLE, compte entier sinon).
async fn realtime_loop(
    runtime: &mut AccountRuntime,
    session: &mut SyncSession,
    mut idle: Option<IdleWatcher>,
) -> Exit {
    let limits = runtime.ctx.provider.descriptor().limits;
    loop {
        let poll = runtime.poll_delay(&limits);
        let result = match wait_for_wake(runtime, &mut idle, poll).await {
            Wake::Shutdown => return Exit::Shutdown,
            Wake::IdleLost => return classify(idle_lost()),
            Wake::InboxChanged => sync_inbox(session).await,
            Wake::Poll => sync_account(session).await,
            Wake::PollChanged => Ok(()),
            Wake::Requested => serve_requests(runtime, session).await,
        };
        if let Err(error) = result {
            return classify(error);
        }
    }
}

/// Une session complète : connexion, état Connected, descente, demandes en attente, puis temps réel jusqu'à la première erreur.
pub async fn run_connected(runtime: &mut AccountRuntime, backoff: &mut Backoff) -> Exit {
    let mut session = match connect(runtime).await {
        Ok(session) => session,
        Err(error) => return classify(error),
    };
    transition(runtime, AccountState::Connected);
    let first = sync_account(&mut session).await;
    let served = match first {
        Ok(()) => serve_requests(runtime, &mut session).await,
        Err(error) => Err(error),
    };
    if let Err(error) = served {
        return classify(error);
    }
    backoff.reset();
    let idle = start_idle(runtime, &session.conn.capabilities).await;
    realtime_loop(runtime, &mut session, idle).await
}
