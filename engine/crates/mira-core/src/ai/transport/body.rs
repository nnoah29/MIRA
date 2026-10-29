//! Lecture bornée d'une réponse non diffusée, comme la liste des modèles.

use mira_vocabulary::{AiError, AiProviderDescriptor};
use reqwest::header::{ACCEPT, HeaderValue};

use super::auth_header::{add_auth_header, header_map};
use super::client::{build_client, endpoint, send_checked};
use super::limits::TransportLimits;
use super::stream::AiStreamBody;
use crate::ports::SecretValue;

/// Lit en GET une ressource du service avec la clé ; le corps est plafonné.
pub async fn get_bounded(
    base_url: &str,
    descriptor: &AiProviderDescriptor,
    key: &SecretValue,
    path: &str,
    limits: &TransportLimits,
) -> Result<Vec<u8>, AiError> {
    let url = endpoint(base_url, path)?;
    let mut headers = header_map(&descriptor.static_headers)?;
    headers
        .entry(ACCEPT)
        .or_insert(HeaderValue::from_static("application/json"));
    add_auth_header(&mut headers, descriptor.auth, key)?;
    let client = build_client(&url, limits)?;
    let response = send_checked(client.get(url).headers(headers), limits).await?;
    read_bounded(AiStreamBody::new(response, limits.idle, limits.max_body)).await
}

/// Lit tout le corps morceau par morceau, en s'arrêtant au-delà du plafond.
async fn read_bounded(mut body: AiStreamBody) -> Result<Vec<u8>, AiError> {
    let mut all = Vec::new();
    while let Some(chunk) = body.next_chunk().await? {
        all.extend_from_slice(&chunk);
    }
    Ok(all)
}
