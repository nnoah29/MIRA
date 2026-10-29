//! Réponse diffusée d'un service d'IA : morceaux lus sous délai d'inactivité, taille totale plafonnée.

use std::fmt;
use std::time::Duration;

use mira_vocabulary::{AiError, AiProviderDescriptor};
use reqwest::Response;
use tokio::time::timeout;

use super::auth_header::{add_auth_header, header_map};
use super::client::{build_client, endpoint, send_checked};
use super::limits::TransportLimits;
use crate::ai::wire::HttpCall;
use crate::ports::SecretValue;

/// Corps d'une réponse en cours ; le détruire ferme la connexion.
pub struct AiStreamBody {
    response: Response,
    idle: Duration,
    max_total: usize,
    received: usize,
}

/// Envoie l'appel de discussion en POST avec la clé et rend le flux de la réponse.
pub async fn post_stream(
    base_url: &str,
    descriptor: &AiProviderDescriptor,
    key: &SecretValue,
    call: &HttpCall,
    limits: &TransportLimits,
) -> Result<AiStreamBody, AiError> {
    let url = endpoint(base_url, &call.path)?;
    let mut headers = header_map(&call.headers)?;
    add_auth_header(&mut headers, descriptor.auth, key)?;
    let client = build_client(&url, limits)?;
    let request = client.post(url).headers(headers).body(call.body.clone());
    let response = send_checked(request, limits).await?;
    Ok(AiStreamBody::new(response, limits.idle, limits.max_stream))
}

impl AiStreamBody {
    /// Enveloppe une réponse avec son délai d'inactivité et son plafond total.
    pub(super) fn new(response: Response, idle: Duration, max_total: usize) -> Self {
        Self {
            response,
            idle,
            max_total,
            received: 0,
        }
    }

    /// Rend le morceau suivant, rien en fin de flux ; silence trop long ou plafond dépassé l'arrêtent.
    pub async fn next_chunk(&mut self) -> Result<Option<Vec<u8>>, AiError> {
        if self.received > self.max_total {
            return Err(AiError::BadResponse);
        }
        let read = timeout(self.idle, self.response.chunk())
            .await
            .map_err(|_| AiError::Unavailable)?;
        let Some(chunk) = read.map_err(|_| AiError::Unavailable)? else {
            return Ok(None);
        };
        self.received = self.received.saturating_add(chunk.len());
        if self.received > self.max_total {
            return Err(AiError::BadResponse);
        }
        Ok(Some(chunk.to_vec()))
    }
}

impl fmt::Debug for AiStreamBody {
    /// Montre seulement la quantité reçue, jamais les en-têtes ni le contenu.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AiStreamBody")
            .field("received", &self.received)
            .finish_non_exhaustive()
    }
}
