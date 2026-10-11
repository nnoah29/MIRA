//! Routage des requêtes HTTP du faux serveur d'autorisation.

use super::context::OAuthContext;
use super::http::request::HttpRequest;
use super::http::response::HttpResponse;
use super::token;

/// Aiguille la requête : seul `POST /token` est servi
pub(crate) fn route(request: &HttpRequest, context: &OAuthContext) -> HttpResponse {
    match (request.method.as_str(), request.path.as_str()) {
        ("POST", "/token") => token::handle(request, context),
        (_, "/token") => HttpResponse::error(405, "invalid_request"),
        _ => HttpResponse::error(404, "not_found"),
    }
}
