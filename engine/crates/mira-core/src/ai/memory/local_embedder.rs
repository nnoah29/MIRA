//! Embedder du modèle local : un serveur dédié, lancé à la demande en mode embeddings et réutilisé tant qu'il vit.

use std::path::PathBuf;
use std::sync::Arc;

use mira_vocabulary::{AiLocalModel, AiLocalRuntime};

use super::embedder::{EmbedFuture, Embedder};
use super::http_embedder::HttpEmbedder;
use crate::ai::local::{LocalRuntimeManager, RuntimeContext};
use crate::ai::transport::TransportLimits;
use crate::ports::SystemResources;

/// Tout ce qu'il faut pour lancer le serveur d'embeddings : gestionnaire propre, fiche, chemins et ressources.
#[derive(Debug)]
pub struct LocalEmbedderSetup {
    pub manager: LocalRuntimeManager,
    pub runtime: AiLocalRuntime,
    pub executable: PathBuf,
    pub run_dir: PathBuf,
    pub models_dir: PathBuf,
    pub resources: Arc<dyn SystemResources>,
    pub model: AiLocalModel,
}

/// Calcule les embeddings avec un serveur local que ce type lance lui-même.
pub struct LocalEmbedder {
    setup: LocalEmbedderSetup,
}

/// Fiche de serveur pour les embeddings : arguments ajoutés, aucun réglage propre aux discussions.
pub fn embedding_runtime(base: &AiLocalRuntime, embedding_args: &[String]) -> AiLocalRuntime {
    let mut runtime = base.clone();
    runtime.server_args.extend(embedding_args.iter().cloned());
    runtime.reasoning_off_args.clear();
    runtime
}

impl LocalEmbedder {
    /// Prépare l'embedder ; aucun processus n'est lancé avant le premier calcul
    pub fn new(setup: LocalEmbedderSetup) -> Self {
        Self { setup }
    }

    /// Arrête le serveur d'embeddings s'il tourne
    pub async fn stop(&self) {
        self.setup.manager.stop().await;
    }
}

impl Embedder for LocalEmbedder {
    /// Identifiant du modèle d'embeddings épinglé
    fn model_id(&self) -> &str {
        &self.setup.model.id
    }

    /// Lance ou réutilise le serveur puis lui envoie les textes
    fn embed<'a>(&'a self, texts: &'a [String]) -> EmbedFuture<'a> {
        Box::pin(async move {
            if texts.is_empty() {
                return Ok(Vec::new());
            }
            let setup = &self.setup;
            let context = RuntimeContext {
                runtime: &setup.runtime,
                executable: setup.executable.clone(),
                run_dir: setup.run_dir.clone(),
                models_dir: setup.models_dir.clone(),
                resources: setup.resources.as_ref(),
            };
            let handle = setup.manager.ensure_running(&context, &setup.model).await?;
            let http = HttpEmbedder::new(
                &handle.root_url,
                handle.api_key,
                setup.model.id.clone(),
                TransportLimits::default(),
            );
            http.embed(texts).await
        })
    }
}

impl std::fmt::Debug for LocalEmbedder {
    /// Ne montre que le modèle, jamais les chemins ni la clé
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LocalEmbedder")
            .field("model", &self.setup.model.id)
            .finish_non_exhaustive()
    }
}
