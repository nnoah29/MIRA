//! Requête et réponse de l'API d'embeddings compatible OpenAI ; la réponse est validée sans jamais citer le corps.

use mira_vocabulary::error::codes::server::AI_BAD_RESPONSE;
use mira_vocabulary::{ErrorCategory, MiraError};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
struct Reply {
    data: Vec<Item>,
}

#[derive(Deserialize)]
struct Item {
    index: usize,
    embedding: Vec<f32>,
}

/// Corps de la requête : le modèle et tous les textes, dans l'ordre
pub fn embeddings_request(model: &str, texts: &[String]) -> Value {
    json!({"model": model, "input": texts})
}

/// Erreur d'une réponse inutilisable, sans aucun extrait du corps
fn bad_response(reason: &str) -> MiraError {
    MiraError::new(
        ErrorCategory::Server,
        AI_BAD_RESPONSE,
        format!("réponse d'embeddings inutilisable : {reason}"),
    )
}

/// Lit la réponse ; rend un vecteur par texte, dans l'ordre de `index`, de dimension unique non nulle
pub fn parse_embeddings(body: &str, expected: usize) -> Result<Vec<Vec<f32>>, MiraError> {
    let reply: Reply = serde_json::from_str(body).map_err(|_| bad_response("format inattendu"))?;
    if reply.data.len() != expected {
        return Err(bad_response("nombre de vecteurs inattendu"));
    }
    let mut items = reply.data;
    items.sort_by_key(|item| item.index);
    let dim = items.first().map_or(0, |item| item.embedding.len());
    let consistent = items.iter().all(|item| item.embedding.len() == dim);
    let finite = items
        .iter()
        .all(|item| item.embedding.iter().all(|v| v.is_finite()));
    if dim == 0 && expected > 0 || !consistent || !finite {
        return Err(bad_response("vecteurs invalides"));
    }
    Ok(items.into_iter().map(|item| item.embedding).collect())
}
