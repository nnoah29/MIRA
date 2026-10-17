//! Fin d'une autorisation OAuth2 (15.2) : redirection attendue, jetons échangés puis rangés au trousseau.

use std::time::Duration;

use mira_vocabulary::error::codes::authentication;
use mira_vocabulary::{AccountState, ErrorCategory, MiraError};

use crate::accounts::{change_account_state, require_active_account};
use crate::api::contract::Response;
use crate::api::engine::Engine;
use crate::auth::{FlowSummary, TokenSet, authorize_until_cancelled};

const REDIRECT_PATIENCE: Duration = Duration::from_secs(300);

/// Attend la redirection, échange le code et range les jetons sous le compte réservé par le flux
pub(crate) async fn handle_complete_oauth(
    engine: &Engine,
    flow: &str,
) -> Result<Response, MiraError> {
    let (mut pending, cancel, summary) = engine.oauth_flows.begin_completion(flow)?;
    let clock = engine.config.clock.as_ref();
    let result = authorize_until_cancelled(&mut pending, &cancel, REDIRECT_PATIENCE, clock).await;
    let stored = result.and_then(|tokens| store_tokens(engine, &summary, &tokens));
    if let Err(error) = stored {
        engine.oauth_flows.remove(flow);
        return Err(error);
    }
    finish(engine, flow, &summary)?;
    Ok(Response::Done)
}

/// Range les jetons sous le compte réservé
fn store_tokens(
    engine: &Engine,
    summary: &FlowSummary,
    tokens: &TokenSet,
) -> Result<(), MiraError> {
    let profile = &engine.config.profile;
    engine
        .vault()
        .store_tokens(profile, &summary.target.account, tokens)
}

/// Termine le flux : garde-le pour l'ajout du compte, ou clôt la réauthentification
fn finish(engine: &Engine, flow: &str, summary: &FlowSummary) -> Result<(), MiraError> {
    if !engine.oauth_flows.mark_completed(flow) {
        discard_orphan_tokens(engine, summary)?;
        return Err(MiraError::new(
            ErrorCategory::Authentication,
            authentication::OAUTH_CANCELLED,
            "autorisation annulée",
        ));
    }
    if !summary.target.reauthentication {
        return Ok(());
    }
    engine.oauth_flows.remove(flow);
    let id = &summary.target.account;
    let account = engine.database.read(|tx| require_active_account(tx, id))?;
    if account.state == AccountState::AuthRequired {
        change_account_state(
            &engine.database,
            &engine.events,
            account,
            AccountState::Offline,
        )?;
    }
    Ok(())
}

/// Efface les jetons d'un flux annulé en cours d'échange, sauf ceux d'un compte qui existe déjà
fn discard_orphan_tokens(engine: &Engine, summary: &FlowSummary) -> Result<(), MiraError> {
    if summary.target.reauthentication {
        return Ok(());
    }
    let profile = &engine.config.profile;
    engine.vault().delete_all(profile, &summary.target.account)
}
