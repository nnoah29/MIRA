//! Ce que le moteur garde de la mémoire pendant sa vie : le gestionnaire du serveur d'embeddings, distinct de celui du chat.

use crate::ai::local::LocalRuntimeManager;

/// Serveur d'embeddings partagé par tous les appels de mémoire du moteur.
#[derive(Debug, Default)]
pub(crate) struct MemoryRuntime {
    manager: LocalRuntimeManager,
}

impl MemoryRuntime {
    /// Rend une poignée sur le gestionnaire, qui reste partagé entre les copies
    pub(crate) fn manager(&self) -> LocalRuntimeManager {
        self.manager.clone()
    }

    /// Arrête le serveur d'embeddings s'il tourne
    pub(crate) async fn stop(&self) {
        self.manager.stop().await;
    }
}
