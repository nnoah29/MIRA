#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Retrait, arrêt et plateforme du modèle local : tout est rendu, rien ne reste.

use mira_core::api::AiRequest;
use mira_core::operations::OperationStatus;

mod local_ops_support;
mod local_support;
mod support_api;
use local_ops_support::OpsFixture;

const BOTH: [&str; 2] = ["linux-x86_64", "linux-aarch64"];

/// Demande le retrait d'un modèle
fn remove(id: &str) -> AiRequest {
    AiRequest::RemoveLocalModel {
        model_id: id.to_string(),
    }
}

/// Retirer le dernier modèle supprime tout le dossier du modèle local et sa connexion
#[tokio::test]
async fn remove_last_model_deletes_files_connection_and_server_folder() {
    let fixture = OpsFixture::start(Some(16_000), &BOTH).await;
    assert_eq!(
        fixture.install_and_wait("mini").await,
        OperationStatus::Finished
    );
    assert!(fixture.root.exists());
    fixture.exec(remove("mini")).await.unwrap();
    assert!(!fixture.root.exists());
    assert!(fixture.connections().await.is_empty());
    assert!(!fixture.status().await.models[0].installed);
}

/// Retirer un modèle quand un autre reste garde le serveur et reporte la connexion sur l'autre
#[tokio::test]
async fn remove_keeps_the_server_when_another_model_remains() {
    let fixture = OpsFixture::start(Some(16_000), &BOTH).await;
    fixture.install_and_wait("mini").await;
    fixture.install_and_wait("big").await;
    fixture.exec(remove("big")).await.unwrap();
    let status = fixture.status().await;
    assert!(status.server_installed && status.models[0].installed && !status.models[1].installed);
    let connections = fixture.connections().await;
    assert_eq!(connections.len(), 1);
    assert_eq!(connections[0].default_model.as_deref(), Some("mini"));
}

/// L'avertissement mémoire ne vise que les modèles qui dépassent la mémoire disponible
#[tokio::test]
async fn memory_warning_flags_models_above_available_memory() {
    let fixture = OpsFixture::start(Some(2_000), &BOTH).await;
    let status = fixture.status().await;
    assert_eq!(status.memory_available_mib, Some(2_000));
    assert!(!status.models[0].memory_warning && status.models[1].memory_warning);
}

/// Sans serveur décrit pour la plateforme, l'état est « non pris en charge » et l'installation refusée
#[tokio::test]
async fn ops_are_refused_when_unsupported() {
    let fixture = OpsFixture::start(Some(16_000), &["plan9-mips"]).await;
    assert!(!fixture.status().await.supported);
    let install = AiRequest::InstallLocalModel {
        model_id: "mini".to_string(),
    };
    let error = fixture.exec(install).await.unwrap_err();
    assert_eq!(
        error.code,
        "incompatible_state.ai_local_unsupported_platform"
    );
    assert!(!fixture.root.exists());
}

/// Retirer un modèle inconnu est refusé sans toucher au disque
#[tokio::test]
async fn removing_an_unknown_model_is_refused() {
    let fixture = OpsFixture::start(Some(16_000), &BOTH).await;
    fixture.install_and_wait("mini").await;
    assert!(fixture.exec(remove("inconnu")).await.is_err());
    assert!(fixture.status().await.models[0].installed);
}
