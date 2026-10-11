//! Rafraîchissement d'un jeton d'accès (RFC 6749 section 6).

use crate::oauth::context::OAuthContext;
use crate::oauth::form::Form;
use crate::oauth::http::response::HttpResponse;

/// Émet un nouveau jeton d'accès si le jeton de rafraîchissement est valide pour ce client
pub(super) fn handle(form: &Form, context: &OAuthContext) -> HttpResponse {
    let (Some(refresh_token), Some(client_id)) = (form.get("refresh_token"), form.get("client_id"))
    else {
        return HttpResponse::error(400, "invalid_request");
    };
    let access = context.shared.with(|state| {
        let grant = state.refresh_grant(refresh_token)?;
        if grant.client_id != client_id {
            return None;
        }
        Some(state.issue_access(&grant.user, &context.credentials))
    });
    match access {
        Some(access) => super::success(&access, None),
        None => HttpResponse::error(400, "invalid_grant"),
    }
}
