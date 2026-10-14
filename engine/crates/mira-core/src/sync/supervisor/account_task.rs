//! Tâche unique d'un compte : session connectée, puis reprise selon la cause de la rupture.

use mira_vocabulary::{AccountState, MiraError};

use super::connected::run_connected;
use super::exit::Exit;
use super::runtime::AccountRuntime;
use super::states::transition;
use crate::sync::realtime::Backoff;

/// Journalise la rupture (code seulement, jamais de secret) et passe le compte dans l'état voulu.
fn record_break(runtime: &mut AccountRuntime, error: &MiraError, state: Option<AccountState>) {
    tracing::info!(code = %error.code, "session de synchronisation interrompue");
    runtime.report.broke(error);
    runtime.requests.fail_all(error);
    if let Some(next) = state {
        transition(runtime, next);
    }
}

/// Boucle de vie d'un compte : se reconnecte avec délai croissant, attend l'utilisateur si l'authentification échoue.
pub async fn run_account(mut runtime: AccountRuntime) {
    let mut backoff = Backoff::new(runtime.config.backoff);
    loop {
        let keep_going = match run_connected(&mut runtime, &mut backoff).await {
            Exit::Shutdown => false,
            Exit::Network(error) => {
                record_break(&mut runtime, &error, Some(AccountState::Offline));
                runtime.pause(backoff.next_delay()).await
            }
            Exit::AuthRequired(error) => {
                record_break(&mut runtime, &error, Some(AccountState::AuthRequired));
                backoff.reset();
                runtime.wait_for_request().await
            }
            Exit::Failed(error) => {
                record_break(&mut runtime, &error, None);
                runtime.pause(backoff.next_delay()).await
            }
        };
        if !keep_going {
            return;
        }
    }
}
