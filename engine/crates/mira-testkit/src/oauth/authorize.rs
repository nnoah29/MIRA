//! Analyse d'une requête d'autorisation : code + PKCE S256 (RFC 6749 4.1.1, RFC 7636 4.3).

use std::io;

use super::error::invalid_input;
use super::form::Form;

/// Paramètres retenus de la requête d'autorisation.
#[derive(Debug, Clone)]
pub(crate) struct AuthorizeRequest {
    pub(crate) client_id: String,
    pub(crate) redirect_uri: String,
    pub(crate) state: Option<String>,
    pub(crate) code_challenge: String,
}

impl AuthorizeRequest {
    /// Analyse l'URL complète ; refuse tout ce qui n'est pas `code` avec S256
    pub(crate) fn parse(url: &str) -> io::Result<Self> {
        let (_, query) = url
            .split_once('?')
            .ok_or_else(|| invalid_input("authorization request has no query"))?;
        let form = Form::parse(query.split('#').next().unwrap_or(query));
        if form.get("response_type") != Some("code") {
            return Err(invalid_input("response_type must be code"));
        }
        if form.get("code_challenge_method") != Some("S256") {
            return Err(invalid_input("code_challenge_method must be S256"));
        }
        Ok(Self {
            client_id: required(&form, "client_id")?,
            redirect_uri: required(&form, "redirect_uri")?,
            state: form.get("state").map(str::to_string),
            code_challenge: required(&form, "code_challenge")?,
        })
    }

    /// Paramètres de la redirection : le couple donné puis `state` s'il a été fourni
    pub(crate) fn callback_params<'a>(
        &'a self,
        key: &'a str,
        value: &'a str,
    ) -> Vec<(&'a str, &'a str)> {
        let mut params = vec![(key, value)];
        if let Some(state) = &self.state {
            params.push(("state", state.as_str()));
        }
        params
    }
}

/// Valeur obligatoire d'un paramètre, erreur sans la valeur sinon
fn required(form: &Form, name: &str) -> io::Result<String> {
    form.get(name)
        .map(str::to_string)
        .ok_or_else(|| invalid_input(&format!("missing parameter {name}")))
}
