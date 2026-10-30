//! Client HTTP d'un échange avec un service d'IA : HTTPS hors boucle locale, sans redirection.

use mira_vocabulary::AiError;
use reqwest::header::{HeaderMap, RETRY_AFTER};
use reqwest::redirect::Policy;
use reqwest::{RequestBuilder, Response, Url};
use tokio::time::timeout;

use super::failure::map_status;
use super::limits::TransportLimits;
use crate::auth::loopback::{is_loopback_host, require_secure_url};

/// Joint le chemin à l'adresse de base ; refuse une URL non sûre ou un chemin qui déborde sur l'hôte.
pub(super) fn endpoint(base_url: &str, path: &str) -> Result<Url, AiError> {
    if !path.starts_with('/') {
        return Err(AiError::UrlInvalid);
    }
    let joined = format!("{}{path}", base_url.trim_end_matches('/'));
    require_secure_url(&joined).map_err(|_| AiError::UrlInvalid)?;
    Url::parse(&joined).map_err(|_| AiError::UrlInvalid)
}

/// Construit un client sans redirection ni en-tête implicite, sans proxy vers la boucle locale.
pub(super) fn build_client(
    url: &Url,
    limits: &TransportLimits,
) -> Result<reqwest::Client, AiError> {
    let mut builder = reqwest::Client::builder()
        .connect_timeout(limits.connect)
        .redirect(Policy::none())
        .default_headers(HeaderMap::new());
    if url.host_str().is_some_and(is_loopback_host) {
        builder = builder.no_proxy();
    }
    builder.build().map_err(|_| AiError::Unavailable)
}

/// Envoie la requête sous le délai du premier octet ; un statut d'échec est traduit sans lire le corps.
pub(super) async fn send_checked(
    request: RequestBuilder,
    limits: &TransportLimits,
) -> Result<Response, AiError> {
    let sent = timeout(limits.first_byte, request.send())
        .await
        .map_err(|_| AiError::Unavailable)?;
    let response = sent.map_err(|_| AiError::Unavailable)?;
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }
    let retry_after = response.headers().get(RETRY_AFTER);
    Err(map_status(
        status.as_u16(),
        retry_after.and_then(|value| value.to_str().ok()),
    ))
}
