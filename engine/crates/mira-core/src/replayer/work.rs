//! Une passe de travail en ligne : connexion, reprise des actions interrompues, rejeu, déconnexion.

use std::sync::Arc;

use mira_vocabulary::error::codes::secrets;
use mira_vocabulary::{Account, AccountState, ErrorCategory, MiraError};

use super::credentials::ProductionCredentials;
use super::session::open_session;
use super::shared::ReplayShared;
use crate::accounts::change_account_state;
use crate::actions::{ReplayContext, recover_interrupted_actions, replay_actions};

/// Se connecte, règle d'abord les actions interrompues (inv. 18, 26) puis rejoue la file du compte
pub(crate) async fn work_online(
    shared: &Arc<ReplayShared>,
    account: &Account,
    recovered: &mut bool,
) -> Result<(), MiraError> {
    let mut session = open_session(shared, account).await?;
    mark_online(shared, account)?;
    let credentials = ProductionCredentials::new(shared);
    let ctx = ReplayContext {
        queue: shared.queue_context(),
        account: &account.id,
        quirks: &session.quirks,
        credentials: &credentials,
        smtp: Some(&session.smtp),
    };
    if !*recovered {
        recover_interrupted_actions(&ctx, &mut session.client).await?;
        *recovered = true;
    }
    replay_actions(&ctx, &mut session.client).await?;
    session.client.logout().await
}

/// Un compte « hors ligne » qui vient de se connecter redevient « connecté »
fn mark_online(shared: &ReplayShared, account: &Account) -> Result<(), MiraError> {
    if account.state != AccountState::Offline {
        return Ok(());
    }
    let state = AccountState::Connected;
    change_account_state(&shared.database, &shared.events, account.clone(), state).map(|_| ())
}

/// Traduit l'échec d'une passe en état du compte : authentification requise, hors ligne, ou simple journal
pub(crate) fn settle_failure(
    shared: &ReplayShared,
    account: &Account,
    error: &MiraError,
) -> Result<(), MiraError> {
    let missing_secret = error.code == secrets::SECRET_MISSING.0;
    let target = match error.category {
        ErrorCategory::Authentication => AccountState::AuthRequired,
        ErrorCategory::Secrets if missing_secret => AccountState::AuthRequired,
        ErrorCategory::Network | ErrorCategory::Tls => AccountState::Offline,
        _ => {
            tracing::warn!(code = %error.code, "rejeu interrompu");
            return Ok(());
        }
    };
    let keep = account.state == target
        || (target == AccountState::Offline && account.state != AccountState::Connected);
    if keep {
        return Ok(());
    }
    change_account_state(&shared.database, &shared.events, account.clone(), target).map(|_| ())
}
