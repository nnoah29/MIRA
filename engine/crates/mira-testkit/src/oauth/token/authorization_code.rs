//! Échange d'un code d'autorisation contre des jetons, avec PKCE (RFC 6749 4.1.3, RFC 7636 4.6).

use crate::oauth::context::OAuthContext;
use crate::oauth::form::Form;
use crate::oauth::http::response::HttpResponse;
use crate::oauth::pkce;

/// Vérifie code, client, redirection et vérificateur PKCE puis émet les jetons
pub(super) fn handle(form: &Form, context: &OAuthContext) -> HttpResponse {
    let (Some(code), Some(redirect_uri), Some(client_id), Some(verifier)) = (
        form.get("code"),
        form.get("redirect_uri"),
        form.get("client_id"),
        form.get("code_verifier"),
    ) else {
        return HttpResponse::error(400, "invalid_request");
    };
    let Some(pending) = context.shared.with(|state| state.take_code(code)) else {
        return HttpResponse::error(400, "invalid_grant");
    };
    let consistent = pending.client_id == client_id
        && pending.redirect_uri == redirect_uri
        && pkce::verifies(verifier, &pending.challenge);
    if !consistent {
        return HttpResponse::error(400, "invalid_grant");
    }
    issue(&pending.user, client_id, context)
}

/// Émet un jeton d'accès et un jeton de rafraîchissement pour l'utilisateur
fn issue(user: &str, client_id: &str, context: &OAuthContext) -> HttpResponse {
    let (access, refresh) = context.shared.with(|state| {
        let access = state.issue_access(user, &context.credentials);
        (access, state.issue_refresh(user, client_id))
    });
    super::success(&access, Some(&refresh))
}
