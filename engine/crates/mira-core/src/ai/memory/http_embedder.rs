//! Embedder pour tout service compatible OpenAI (`/v1/embeddings`) : serveur local ou fournisseur choisi.

use mira_vocabulary::AuthScheme;

use super::embedder::{EmbedFuture, Embedder};
use super::embeddings_wire::{embeddings_request, parse_embeddings};
use crate::ai::transport::{TransportLimits, post_bounded};
use crate::ports::SecretValue;

const EMBEDDINGS_PATH: &str = "/v1/embeddings";

/// Client d'un service d'embeddings : adresse de base, clé, modèle et plafonds de l'échange.
pub struct HttpEmbedder {
    base_url: String,
    key: SecretValue,
    model_id: String,
    limits: TransportLimits,
}

impl HttpEmbedder {
    /// Prépare le client ; rien n'est envoyé avant le premier calcul
    pub fn new(
        base_url: &str,
        key: SecretValue,
        model_id: String,
        limits: TransportLimits,
    ) -> Self {
        Self {
            base_url: base_url.to_string(),
            key,
            model_id,
            limits,
        }
    }
}

impl Embedder for HttpEmbedder {
    /// Identifiant du modèle demandé au service
    fn model_id(&self) -> &str {
        &self.model_id
    }

    /// Envoie les textes en une requête et valide les vecteurs reçus
    fn embed<'a>(&'a self, texts: &'a [String]) -> EmbedFuture<'a> {
        Box::pin(async move {
            if texts.is_empty() {
                return Ok(Vec::new());
            }
            let body = embeddings_request(&self.model_id, texts).to_string();
            let bytes = post_bounded(
                &self.base_url,
                AuthScheme::Bearer,
                &self.key,
                EMBEDDINGS_PATH,
                body,
                &self.limits,
            )
            .await?;
            parse_embeddings(&String::from_utf8_lossy(&bytes), texts.len())
        })
    }
}

impl std::fmt::Debug for HttpEmbedder {
    /// Montre l'adresse et le modèle, jamais la clé
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("HttpEmbedder")
            .field("base_url", &self.base_url)
            .field("model_id", &self.model_id)
            .finish_non_exhaustive()
    }
}
