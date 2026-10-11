//! Lectures par la façade, pour vérifier l'état après une opération.

use mira_core::api::{AccountRequest, Engine, EventRequest, IdentityRequest, Request, Response};
use mira_vocabulary::{Account, AccountId, AccountState, EventFilter, EventKind, Identity};

/// Identités d'un compte, l'identité par défaut en premier
pub async fn identities(engine: &Engine, account: &AccountId) -> Vec<Identity> {
    let request = Request::Identity(IdentityRequest::ListIdentities {
        account: account.clone(),
    });
    match engine.execute(request).await.unwrap() {
        Response::Identities(list) => list,
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Tous les comptes du profil
pub async fn accounts(engine: &Engine) -> Vec<Account> {
    match engine
        .execute(Request::Account(AccountRequest::ListAccounts))
        .await
        .unwrap()
    {
        Response::Accounts(list) => list,
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Nature de tous les événements émis depuis le début, dans l'ordre
pub async fn event_kinds(engine: &Engine) -> Vec<EventKind> {
    let request = Request::Event(EventRequest::GetEvents {
        filter: EventFilter::default(),
        limit: 1000,
    });
    match engine.execute(request).await.unwrap() {
        Response::Events(events) => events.into_iter().map(|event| event.kind).collect(),
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// États successifs annoncés par `AccountStateChanged`, dans l'ordre
pub async fn state_changes(engine: &Engine) -> Vec<AccountState> {
    event_kinds(engine)
        .await
        .into_iter()
        .filter_map(|kind| match kind {
            EventKind::AccountStateChanged { state, .. } => Some(state),
            _ => None,
        })
        .collect()
}
