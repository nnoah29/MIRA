//! Ajout d'un compte OAuth2 en un seul processus : le flux vit en mémoire du moteur.

use std::time::Duration;

use mira_assembly::api::contract::OAuthStart;
use mira_assembly::api::{AccountRequest, AuthRequest, CredentialInput, Engine, Request, Response};
use mira_assembly::mira_vocabulary::error::codes::{internal, network};
use mira_assembly::mira_vocabulary::{ErrorCategory, MiraError, ProviderId};

use super::add_oauth::AddOauthArgs;
use super::sync_wait::settle_first_sync;
use crate::cli_io::CliIo;

/// Ouvre le flux OAuth2 et rend son identifiant et l'adresse d'autorisation.
async fn begin(engine: &Engine, args: &AddOauthArgs) -> Result<OAuthStart, MiraError> {
    let start = AuthRequest::StartOAuth {
        address: args.address.clone(),
        provider: args.provider.as_deref().map(ProviderId::new),
    };
    match engine.execute(Request::Auth(start)).await? {
        Response::OAuthStarted(started) => Ok(started),
        _ => Err(unexpected("réponse inattendue au démarrage du flux OAuth")),
    }
}

/// Erreur interne pour une réponse que le contrat ne prévoit pas.
fn unexpected(message: &str) -> MiraError {
    MiraError::new(ErrorCategory::Internal, internal::BUG, message)
}

/// Dit à l'utilisateur où autoriser MIRA ; l'adresse part sur stderr pour laisser stdout au résultat.
fn announce(io: &mut CliIo<'_>, started: &OAuthStart) {
    let _ = writeln!(
        io.err,
        "Ouvrez cette adresse dans un navigateur pour autoriser MIRA :\n{}\nEn attente de la redirection...",
        started.authorize_url
    );
    let _ = io.err.flush();
}

/// Attend la redirection au plus `seconds` secondes ; passé ce délai, le flux est annulé.
async fn complete_within(engine: &Engine, flow: &str, seconds: u64) -> Result<(), MiraError> {
    let complete = AuthRequest::CompleteOAuth {
        flow: flow.to_string(),
    };
    let waiting = engine.execute(Request::Auth(complete));
    if let Ok(outcome) = tokio::time::timeout(Duration::from_secs(seconds), waiting).await {
        return outcome.map(|_| ());
    }
    let cancel = AuthRequest::CancelOAuth {
        flow: flow.to_string(),
    };
    let _ = engine.execute(Request::Auth(cancel)).await;
    Err(MiraError::new(
        ErrorCategory::Network,
        network::TIMEOUT,
        format!("autorisation non reçue après {seconds} s ; flux annulé"),
    ))
}

/// Remplace le flux de la requête d'ajout par le flux autorisé.
fn with_flow(request: Request, flow: String) -> Result<Request, MiraError> {
    match request {
        Request::Account(AccountRequest::AddAccount {
            draft,
            idempotency_key,
            ..
        }) => Ok(Request::Account(AccountRequest::AddAccount {
            draft,
            credential: CredentialInput::OAuthFlow { flow },
            idempotency_key,
        })),
        _ => Err(unexpected("requête d'ajout OAuth mal formée")),
    }
}

/// Démarre le flux, attend l'autorisation, ajoute le compte, puis attend sa première synchro si demandé.
pub async fn login(
    engine: &Engine,
    args: &AddOauthArgs,
    request: Request,
    io: &mut CliIo<'_>,
) -> Result<Response, MiraError> {
    let started = begin(engine, args).await?;
    announce(io, &started);
    complete_within(engine, &started.flow, args.sync.timeout).await?;
    let added = engine.execute(with_flow(request, started.flow)?).await?;
    settle_first_sync(engine, added, &args.sync).await
}
