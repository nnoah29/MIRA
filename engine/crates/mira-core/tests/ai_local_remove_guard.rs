#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Garde du retrait : aucun retrait pendant une installation, pour ne pas effacer son téléchargement.

use mira_core::api::AiRequest;
use mira_core::operations::OperationStatus;

mod local_ops_support;
mod local_support;
mod support_api;
use local_ops_support::OpsFixture;

const BOTH: [&str; 2] = ["linux-x86_64", "linux-aarch64"];

/// Retirer un modèle pendant l'installation d'un autre est refusé : le téléchargement en cours est intact
#[tokio::test]
async fn remove_is_refused_while_another_model_installs() {
    let fixture = OpsFixture::start(Some(16_000), &BOTH).await;
    fixture.install_and_wait("mini").await;
    let request = AiRequest::InstallLocalModel {
        model_id: "big".to_string(),
    };
    let mira_core::api::Response::Operation(info) = fixture.exec(request).await.unwrap() else {
        panic!()
    };
    let error = fixture
        .exec(AiRequest::RemoveLocalModel {
            model_id: "mini".to_string(),
        })
        .await
        .unwrap_err();
    assert_eq!(error.code, "incompatible_state.ai_local_busy");
    assert_eq!(fixture.wait_end(&info.id).await, OperationStatus::Finished);
    assert!(fixture.status().await.models.iter().all(|m| m.installed));
}
