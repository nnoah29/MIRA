//! Interface d'un calculateur d'embeddings : des textes en entrée, un vecteur par texte en sortie.

use std::future::Future;
use std::pin::Pin;

use mira_vocabulary::MiraError;

/// Résultat différé d'un calcul d'embeddings.
pub type EmbedFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Vec<Vec<f32>>, MiraError>> + Send + 'a>>;

/// Calcule les vecteurs de textes ; le nom du modèle sert à ne comparer que des vecteurs comparables.
pub trait Embedder: Send + Sync {
    /// Identifiant du modèle qui produit les vecteurs
    fn model_id(&self) -> &str;

    /// Un vecteur par texte, dans l'ordre ; sans texte, une liste vide sans aucune requête
    fn embed<'a>(&'a self, texts: &'a [String]) -> EmbedFuture<'a>;
}
