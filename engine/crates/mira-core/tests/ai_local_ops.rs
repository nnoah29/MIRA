#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Opérations du modèle local : état sans effet, installation, refus d'une seconde, annulation.

use mira_core::api::{AiRequest, OperationRequest, Request, Response};
use mira_core::operations::OperationStatus;

mod local_ops_support;
mod local_support;
mod support_api;
use local_ops_support::OpsFixture;

const BOTH: [&str; 2] = ["linux-x86_64", "linux-aarch64"];

/// L'état liste les deux modèles de la fiche sans rien créer, télécharger ni lancer
#[tokio::test]
async fn status_changes_nothing_on_disk_and_lists_the_models() {
    let fixture = OpsFixture::start(Some(16_000), &BOTH).await;
    let status = fixture.status().await;
    assert!(status.supported && !status.server_installed && status.installing.is_none());
    assert_eq!(status.models.len(), 2);
    assert!(status.models.iter().all(|m| !m.installed));
    assert!(!fixture.root.exists());
    assert!(fixture.server_host.requests().is_empty() && fixture.mini_host.requests().is_empty());
}

/// L'installation télécharge le serveur puis le modèle et crée la connexion avec ce modèle par défaut
#[tokio::test]
async fn install_downloads_server_then_model_and_creates_the_connection() {
    let fixture = OpsFixture::start(Some(16_000), &BOTH).await;
    assert_eq!(
        fixture.install_and_wait("mini").await,
        OperationStatus::Finished
    );
    let status = fixture.status().await;
    assert!(status.server_installed && status.models[0].installed && !status.models[1].installed);
    assert!(status.installing.is_none());
    let connections = fixture.connections().await;
    assert_eq!(connections.len(), 1);
    assert_eq!(connections[0].default_model.as_deref(), Some("mini"));
    assert!(!connections[0].has_key && connections[0].consent_at.is_none());
}

/// Une seconde installation lancée pendant la première est refusée
#[tokio::test]
async fn second_install_while_one_runs_is_refused() {
    let fixture = OpsFixture::start(Some(16_000), &BOTH).await;
    let first = AiRequest::InstallLocalModel {
        model_id: "big".to_string(),
    };
    let Response::Operation(info) = fixture.exec(first).await.unwrap() else {
        panic!()
    };
    let second = AiRequest::InstallLocalModel {
        model_id: "mini".to_string(),
    };
    let error = fixture.exec(second).await.unwrap_err();
    assert_eq!(error.code, "incompatible_state.ai_local_busy");
    assert_eq!(fixture.wait_end(&info.id).await, OperationStatus::Finished);
}

/// L'annulation de l'opération la marque annulée, libère l'emplacement et ne laisse aucun modèle
#[tokio::test]
async fn cancel_install_marks_the_operation_cancelled_and_frees_the_slot() {
    let fixture = OpsFixture::start(Some(16_000), &BOTH).await;
    let request = AiRequest::InstallLocalModel {
        model_id: "big".to_string(),
    };
    let Response::Operation(info) = fixture.exec(request).await.unwrap() else {
        panic!()
    };
    let cancel = Request::Operation(OperationRequest::CancelOperation {
        operation: info.id.clone(),
    });
    fixture.engine.execute(cancel).await.unwrap();
    assert_eq!(fixture.wait_end(&info.id).await, OperationStatus::Cancelled);
    assert!(fixture.status().await.installing.is_none());
    assert!(!fixture.status().await.models[1].installed);
}

/// Un modèle inconnu est refusé avant toute opération
#[tokio::test]
async fn install_of_an_unknown_model_is_refused() {
    let fixture = OpsFixture::start(Some(16_000), &BOTH).await;
    let request = AiRequest::InstallLocalModel {
        model_id: "inconnu".to_string(),
    };
    assert!(fixture.exec(request).await.is_err());
    assert!(fixture.status().await.installing.is_none());
}

/// Réinstaller un modèle déjà présent ne retélécharge rien
#[tokio::test]
async fn installing_an_installed_model_again_downloads_nothing() {
    let fixture = OpsFixture::start(Some(16_000), &BOTH).await;
    fixture.install_and_wait("mini").await;
    let (server, model) = (
        fixture.server_host.requests().len(),
        fixture.mini_host.requests().len(),
    );
    assert_eq!(
        fixture.install_and_wait("mini").await,
        OperationStatus::Finished
    );
    assert_eq!(fixture.server_host.requests().len(), server);
    assert_eq!(fixture.mini_host.requests().len(), model);
}
