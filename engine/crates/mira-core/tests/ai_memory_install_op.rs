#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Installation du modèle d'embeddings de la mémoire : serveur puis modèle, sans connexion de chat, annulable.

use mira_core::api::{AiRequest, OperationRequest, Request, Response};
use mira_core::operations::OperationStatus;

mod local_ops_support;
mod local_support;
mod support_api;
use local_ops_support::OpsFixture;

const BOTH: [&str; 2] = ["linux-x86_64", "linux-aarch64"];

/// Lance l'installation de la mémoire et rend la fiche de l'opération
async fn start_install(fixture: &OpsFixture) -> mira_core::operations::OperationInfo {
    match fixture.exec(AiRequest::InstallMemoryModel).await.unwrap() {
        Response::Operation(info) => info,
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Le serveur puis le modèle sont installés, sous `ai-memory/`, sans créer de connexion de chat
#[tokio::test]
async fn install_downloads_server_then_embedding_model_without_a_connection() {
    let fixture = OpsFixture::start(Some(16_000), &BOTH).await;
    let info = start_install(&fixture).await;
    assert_eq!(fixture.wait_end(&info.id).await, OperationStatus::Finished);
    let model = fixture
        .root
        .parent()
        .unwrap()
        .join("ai-memory/embeddings/embed.bin");
    assert!(model.is_file());
    assert!(fixture.status().await.server_installed);
    assert!(fixture.connections().await.is_empty());
}

/// Une installation du modèle de chat en cours fait refuser celle de la mémoire
#[tokio::test]
async fn memory_install_is_refused_while_a_chat_install_runs() {
    let fixture = OpsFixture::start(Some(16_000), &BOTH).await;
    let chat = AiRequest::InstallLocalModel {
        model_id: "big".to_string(),
    };
    let Response::Operation(first) = fixture.exec(chat).await.unwrap() else {
        panic!()
    };
    let error = fixture
        .exec(AiRequest::InstallMemoryModel)
        .await
        .unwrap_err();
    assert_eq!(error.code, "incompatible_state.ai_local_busy");
    assert_eq!(fixture.wait_end(&first.id).await, OperationStatus::Finished);
}

/// L'annulation marque l'opération annulée et libère l'emplacement
#[tokio::test]
async fn cancelled_memory_install_frees_the_slot() {
    let fixture = OpsFixture::start(Some(16_000), &BOTH).await;
    let info = start_install(&fixture).await;
    let cancel = Request::Operation(OperationRequest::CancelOperation {
        operation: info.id.clone(),
    });
    fixture.engine.execute(cancel).await.unwrap();
    assert_eq!(fixture.wait_end(&info.id).await, OperationStatus::Cancelled);
    assert!(fixture.status().await.installing.is_none());
}

/// Une plateforme que la fiche ne décrit pas est refusée avant toute opération
#[tokio::test]
async fn memory_install_on_an_undescribed_platform_is_refused() {
    let fixture = OpsFixture::start(Some(16_000), &[]).await;
    let error = fixture
        .exec(AiRequest::InstallMemoryModel)
        .await
        .unwrap_err();
    assert_eq!(
        error.code,
        "incompatible_state.ai_local_unsupported_platform"
    );
}

/// Réinstaller ne retélécharge rien
#[tokio::test]
async fn installing_again_downloads_nothing() {
    let fixture = OpsFixture::start(Some(16_000), &BOTH).await;
    let info = start_install(&fixture).await;
    fixture.wait_end(&info.id).await;
    let (server, model) = (
        fixture.server_host.requests().len(),
        fixture.mini_host.requests().len(),
    );
    let again = start_install(&fixture).await;
    assert_eq!(fixture.wait_end(&again.id).await, OperationStatus::Finished);
    assert_eq!(fixture.server_host.requests().len(), server);
    assert_eq!(fixture.mini_host.requests().len(), model);
}
