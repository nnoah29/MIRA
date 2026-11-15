//! Identifiant du modèle d'embeddings courant, tel qu'inscrit dans les souvenirs : sert au statut et au mode fournisseur.

use mira_vocabulary::{AiProviderId, MemoryEmbedder, MemorySettings};

use crate::ai::local::local_descriptor;
use crate::api::Engine;

/// Identifiant qualifié `<fournisseur>:<modèle>`, pour ne jamais mélanger deux services qui nomment leurs modèles pareil
pub(super) fn qualified_model_id(provider: &AiProviderId, model: &str) -> String {
    format!("{}:{model}", provider.0)
}

/// Modèle d'embeddings des réglages courants, s'il se déduit : fiche locale, ou connexion et modèle choisis
pub(crate) fn current_model_id(engine: &Engine, settings: &MemorySettings) -> Option<String> {
    match &settings.embedder {
        MemoryEmbedder::Local => {
            let runtime = local_descriptor(engine)?.local_runtime?;
            Some(runtime.embedding?.model.id)
        }
        MemoryEmbedder::Provider { connection, model } => {
            let found = engine
                .database
                .read(|tx| tx.require_ai_connection(connection));
            found.ok().map(|c| qualified_model_id(&c.provider, model))
        }
    }
}
