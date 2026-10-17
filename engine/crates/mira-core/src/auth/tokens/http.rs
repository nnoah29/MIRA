//! Requête HTTP vers le point d'accès de jetons (encapsule `reqwest`, rustls).

use std::time::Duration;

use mira_vocabulary::error::codes::{internal, server};
use mira_vocabulary::{ErrorCategory, MiraError};
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use reqwest::redirect::Policy;

use super::failure::map_http_error;
use crate::auth::form::build_query;
use crate::auth::loopback::{is_loopback_host, require_secure_url};

const HTTP_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_BODY: usize = 1 << 20;

/// Envoie un formulaire en POST et renvoie le statut HTTP et le corps de la réponse
pub(super) async fn post_form(
    token_url: &str,
    pairs: &[(&str, &str)],
) -> Result<(u16, Vec<u8>), MiraError> {
    require_secure_url(token_url)?;
    let response = http_client(token_url)?
        .post(token_url)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(ACCEPT, "application/json")
        .body(build_query(pairs))
        .send()
        .await
        .map_err(|error| map_http_error(&error))?;
    let status = response.status().as_u16();
    let body = read_bounded_body(response).await?;
    Ok((status, body))
}

/// Lit le corps par morceaux et s'arrête dès que `MAX_BODY` est dépassé
async fn read_bounded_body(mut response: reqwest::Response) -> Result<Vec<u8>, MiraError> {
    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|error| map_http_error(&error))?
    {
        if body.len() + chunk.len() > MAX_BODY {
            return Err(MiraError::new(
                ErrorCategory::Server,
                server::PROTOCOL,
                "réponse de jetons trop volumineuse",
            ));
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

/// Client sans redirection ; le proxy système est ignoré pour la boucle locale
fn http_client(token_url: &str) -> Result<reqwest::Client, MiraError> {
    let mut builder = reqwest::Client::builder()
        .timeout(HTTP_TIMEOUT)
        .redirect(Policy::none());
    let host = reqwest::Url::parse(token_url)
        .ok()
        .and_then(|url| url.host_str().map(str::to_string));
    if host.is_some_and(|name| is_loopback_host(&name)) {
        builder = builder.no_proxy();
    }
    builder.build().map_err(|_| {
        MiraError::new(
            ErrorCategory::Internal,
            internal::BUG,
            "client HTTP indisponible",
        )
    })
}
