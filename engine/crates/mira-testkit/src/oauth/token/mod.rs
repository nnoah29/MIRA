//! Point d'accès `POST /token` (RFC 6749 section 3.2) : aiguillage par `grant_type`.

mod authorization_code;
mod refresh;

use serde_json::json;

use super::context::OAuthContext;
use super::form::Form;
use super::http::request::HttpRequest;
use super::http::response::HttpResponse;

/// Traite une requête de jeton selon son type d'autorisation
pub(crate) fn handle(request: &HttpRequest, context: &OAuthContext) -> HttpResponse {
    context.shared.with(|state| state.count_token_request());
    if !is_form(request) {
        return HttpResponse::error(400, "invalid_request");
    }
    let form = Form::parse(&String::from_utf8_lossy(&request.body));
    let accepted = context
        .shared
        .with(|state| state.client_secret_accepted(form.get("client_secret")));
    if !accepted {
        return HttpResponse::error(401, "invalid_client");
    }
    match form.get("grant_type") {
        Some("authorization_code") => authorization_code::handle(&form, context),
        Some("refresh_token") => refresh::handle(&form, context),
        Some(_) => HttpResponse::error(400, "unsupported_grant_type"),
        None => HttpResponse::error(400, "invalid_request"),
    }
}

/// Indique si la requête déclare un corps `application/x-www-form-urlencoded`
fn is_form(request: &HttpRequest) -> bool {
    request.header("content-type").is_some_and(|value| {
        value
            .to_ascii_lowercase()
            .starts_with("application/x-www-form-urlencoded")
    })
}

/// Réponse 200 avec le jeton d'accès et, si présent, le jeton de rafraîchissement
pub(super) fn success(access_token: &str, refresh_token: Option<&str>) -> HttpResponse {
    let mut body = json!({
        "access_token": access_token,
        "expires_in": 3600,
        "token_type": "Bearer",
    });
    if let Some(refresh_token) = refresh_token {
        body["refresh_token"] = json!(refresh_token);
    }
    HttpResponse::json(200, &body)
}
