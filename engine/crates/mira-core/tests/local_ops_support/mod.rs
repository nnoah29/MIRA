#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]
//! Moteur ouvert sur un fournisseur local de test dont le serveur et les modèles se téléchargent chez un faux hébergeur.

mod build;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use mira_core::api::Engine;
use mira_core::api::{AiRequest, AiResponse, OperationRequest, Request, Response};
use mira_core::operations::OperationStatus;
use mira_vocabulary::{AiConnection, LocalRuntimeStatus, MiraError, OperationId};

use crate::local_support::file_host::FileHost;

/// Moteur de test et ses trois hébergeurs (serveur, petit modèle, grand modèle)
pub struct OpsFixture {
    pub home: tempfile::TempDir,
    pub engine: Arc<Engine>,
    pub root: PathBuf,
    pub log: PathBuf,
    pub server_host: FileHost,
    pub mini_host: FileHost,
    pub big_host: FileHost,
}

impl OpsFixture {
    /// Ouvre le moteur ; la mémoire disponible et les plateformes décrites sont réglables
    pub async fn start(memory_mib: Option<u64>, platforms: &[&str]) -> Self {
        build::build(memory_mib, platforms).await
    }

    /// Exécute une requête du domaine IA
    pub async fn exec(&self, request: AiRequest) -> Result<Response, MiraError> {
        self.engine.execute(Request::Ai(request)).await
    }

    /// État du modèle local
    pub async fn status(&self) -> LocalRuntimeStatus {
        match self.exec(AiRequest::GetLocalRuntimeStatus).await.unwrap() {
            Response::Ai(AiResponse::LocalRuntime(status)) => status,
            other => panic!("réponse inattendue : {other:?}"),
        }
    }

    /// Lance l'installation et attend la fin de l'opération (20 s au plus) ; rend son statut final
    pub async fn install_and_wait(&self, model_id: &str) -> OperationStatus {
        let request = AiRequest::InstallLocalModel {
            model_id: model_id.to_string(),
        };
        match self.exec(request).await.unwrap() {
            Response::Operation(info) => self.wait_end(&info.id).await,
            other => panic!("réponse inattendue : {other:?}"),
        }
    }

    /// Attend que l'opération ne soit plus en cours
    pub async fn wait_end(&self, operation: &OperationId) -> OperationStatus {
        for _ in 0..2000 {
            let request = Request::Operation(OperationRequest::GetOperation {
                operation: operation.clone(),
            });
            let Response::Operation(info) = self.engine.execute(request).await.unwrap() else {
                panic!("réponse inattendue");
            };
            if !matches!(info.status, OperationStatus::Running { .. }) {
                return info.status;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("opération non terminée");
    }

    /// Connexions d'IA du profil
    pub async fn connections(&self) -> Vec<AiConnection> {
        match self.exec(AiRequest::ListAiConnections).await.unwrap() {
            Response::Ai(AiResponse::Connections(list)) => list,
            other => panic!("réponse inattendue : {other:?}"),
        }
    }
}
