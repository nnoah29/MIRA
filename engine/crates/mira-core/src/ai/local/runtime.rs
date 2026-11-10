//! Gestionnaire du serveur local : un seul processus à la fois, lancé à la demande, réutilisé tant qu'il vit.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use mira_vocabulary::{AiLocalModel, AiLocalRuntime, MiraError};
use tokio::sync::Mutex;

use super::launch::launch;
use super::runtime_state::{State, reuse, spawn_idle_monitor, stop_locked};
use crate::ports::{SecretValue, SystemResources};

/// Ce qu'il faut savoir du système de fichiers et de la machine pour lancer un modèle.
#[derive(Debug)]
pub struct RuntimeContext<'a> {
    pub runtime: &'a AiLocalRuntime,
    pub executable: PathBuf,
    pub run_dir: PathBuf,
    pub models_dir: PathBuf,
    pub resources: &'a dyn SystemResources,
}

/// Adresses et clé d'un serveur en marche ; la clé n'est jamais affichée.
#[derive(Debug, Clone)]
pub struct LocalRuntimeHandle {
    pub root_url: String,
    pub base_url: String,
    pub api_key: SecretValue,
}

/// Gestionnaire du serveur local, partagé par le moteur.
#[derive(Clone, Default)]
pub struct LocalRuntimeManager {
    inner: Arc<Mutex<State>>,
}

impl LocalRuntimeManager {
    /// Crée un gestionnaire sans serveur en marche
    pub fn new() -> Self {
        Self::default()
    }

    /// Rend le serveur de ce modèle, en le lançant (ou le relançant) si besoin ; un seul lancement à la fois.
    pub async fn ensure_running(
        &self,
        ctx: &RuntimeContext<'_>,
        model: &AiLocalModel,
    ) -> Result<LocalRuntimeHandle, MiraError> {
        let mut state = self.inner.lock().await;
        if let Some(handle) = reuse(&mut state, &model.id) {
            return Ok(handle);
        }
        stop_locked(&mut state).await;
        let running = launch(ctx, model).await?;
        let handle = running.handle();
        state.running = Some(running);
        state.generation += 1;
        let idle = Duration::from_secs(ctx.runtime.idle_stop_seconds.max(1));
        spawn_idle_monitor(self.inner.clone(), state.generation, idle);
        Ok(handle)
    }

    /// Arrête le serveur s'il tourne ; sans effet sinon.
    pub async fn stop(&self) {
        let mut state = self.inner.lock().await;
        state.generation += 1;
        stop_locked(&mut state).await;
    }

    /// Identifiant du modèle en marche, s'il y en a un et que son processus vit encore.
    pub async fn running_model(&self) -> Option<String> {
        let mut state = self.inner.lock().await;
        let running = state.running.as_mut()?;
        matches!(running.child.try_wait(), Ok(None)).then(|| running.model_id.clone())
    }
}

impl std::fmt::Debug for LocalRuntimeManager {
    /// Montre seulement le type : l'état porte la clé du serveur
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LocalRuntimeManager")
            .finish_non_exhaustive()
    }
}
