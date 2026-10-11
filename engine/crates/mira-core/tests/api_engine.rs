#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Façade : ouverture, version du contrat, verrou de profil, requêtes de base (ch. 12).

mod support_api;

use mira_core::api::{Engine, EngineRequest, ProviderRequest, Request, Response};
use mira_vocabulary::{CONTRACT_VERSION, ContractVersion, EventFilter, EventKind};

/// Un client d'une version majeure différente est refusé à l'ouverture
#[test]
fn invariant_33_incompatible_client_version_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let client = ContractVersion {
        major: CONTRACT_VERSION.major + 1,
        minor: 0,
    };
    let error = Engine::open(support_api::config(home.path()), client).unwrap_err();
    assert_eq!(error.code, "invalid_input.contract_version");
}

/// Un second moteur sur le même profil est refusé tant que le premier est ouvert (D-27)
#[test]
fn second_engine_on_same_profile_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let first = support_api::open(home.path());
    let error = Engine::open(support_api::config(home.path()), CONTRACT_VERSION).unwrap_err();
    assert_eq!(error.code, "incompatible_state.profile_locked");
    first.shutdown().unwrap();
    support_api::open(home.path()).shutdown().unwrap();
}

/// La version du contrat et l'état sont lisibles sans réseau
#[tokio::test]
async fn contract_info_and_status_are_local_reads() {
    let home = tempfile::tempdir().unwrap();
    let engine = support_api::open(home.path());
    let info = engine
        .execute(Request::Engine(EngineRequest::GetContractInfo))
        .await
        .unwrap();
    assert!(matches!(info, Response::ContractInfo(ref c) if c.version == CONTRACT_VERSION));
    let status = engine
        .execute(Request::Engine(EngineRequest::GetStatus))
        .await
        .unwrap();
    assert!(matches!(status, Response::Status(ref s) if s.accounts.is_empty()));
}

/// La détection passe par le registre injecté et une adresse invalide donne un code stable
#[tokio::test]
async fn provider_detection_through_the_contract() {
    let home = tempfile::tempdir().unwrap();
    let engine = support_api::open(home.path());
    let detect = |address: &str| {
        Request::Provider(ProviderRequest::DetectProvider {
            address: address.into(),
        })
    };
    let found = engine.execute(detect("bob@labels.test")).await.unwrap();
    assert!(matches!(found, Response::Provider(ref p) if p.id.as_str() == "fake_labels"));
    let error = engine.execute(detect("pas une adresse")).await.unwrap_err();
    assert_eq!(error.code, "invalid_input.address");
}

/// L'ouverture émet `EngineStarted`, relisible par numéro
#[tokio::test]
async fn opening_emits_engine_started() {
    let home = tempfile::tempdir().unwrap();
    let engine = support_api::open(home.path());
    let mut subscription = engine.subscribe(EventFilter::default());
    let first = subscription.next().await.unwrap();
    assert_eq!(first.kind, EventKind::EngineStarted);
}
