//! Exemples de réponses des domaines moteur, providers, comptes, identités, authentification, opérations et événements.

use mira_core::api::contract::{AccountPreparation, OAuthStart};
use mira_core::api::{AccountStatus, ContractInfo, EngineStatus, Response};
use mira_core::operations::{OperationInfo, OperationStatus};
use mira_vocabulary::{
    CONTRACT_VERSION, EmailAddress, Event, EventKind, EventSeq, OperationId, ProfileId, Provider,
    Timestamp,
};

use crate::provider_fakes::FakeFoldersProvider;
use crate::support::accounts::{sample_account, sample_identity};

/// Version du contrat et état global d'exemple
fn engine_info_responses() -> Vec<Response> {
    let status = EngineStatus {
        profile: ProfileId::from_raw("default"),
        data_directory: "/data".to_string(),
        accounts: vec![AccountStatus {
            account: sample_account("a"),
            pending_actions: 2,
        }],
    };
    let info = ContractInfo {
        version: CONTRACT_VERSION,
        engine_capabilities: vec!["imap".to_string()],
    };
    vec![
        Response::Done,
        Response::ContractInfo(info),
        Response::Status(status),
    ]
}

/// Réponses des opérations et des événements
fn engine_responses() -> Vec<Response> {
    let operation = OperationInfo {
        id: OperationId::from_raw("o"),
        kind: "fetch_message_body".to_string(),
        status: OperationStatus::Running { done: 1, total: 2 },
    };
    let event = Event {
        seq: EventSeq(1),
        at: Timestamp::from_millis(1),
        kind: EventKind::EngineStarted,
    };
    let mut all = engine_info_responses();
    all.push(Response::Operation(operation));
    all.push(Response::Events(vec![event]));
    all
}

/// Réponses des providers, des comptes, des identités et de l'authentification
fn account_responses() -> Vec<Response> {
    let provider = FakeFoldersProvider::new("p", "b.c");
    let address = EmailAddress::parse("a@b.c").unwrap();
    let preparation = AccountPreparation {
        provider: provider.descriptor(),
        servers: provider.servers_for(&address),
    };
    let start = OAuthStart {
        flow: "f".to_string(),
        authorize_url: "http://127.0.0.1:1/authorize".to_string(),
    };
    vec![
        Response::Providers(vec![provider.descriptor()]),
        Response::Provider(provider.descriptor()),
        Response::AccountPreparation(preparation),
        Response::Accounts(vec![sample_account("a")]),
        Response::Account(sample_account("a")),
        Response::Identities(vec![sample_identity("i", "a", true)]),
        Response::Identity(sample_identity("i", "a", true)),
        Response::OAuthStarted(start),
    ]
}

/// Un exemple par réponse des domaines existants avant la lecture
pub fn responses() -> Vec<Response> {
    let mut all = engine_responses();
    all.extend(account_responses());
    all
}
