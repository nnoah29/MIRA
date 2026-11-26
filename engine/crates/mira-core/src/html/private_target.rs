//! Cible d'une ressource distante : une machine locale ou privée n'est jamais jointe, même avec l'accord de l'utilisateur.

use url::Url;

use crate::mime::is_private_host;

/// Vrai si l'adresse normalisée (http, https ou relative au protocole) vise une machine locale, privée ou réservée.
pub(super) fn targets_private_host(normalized: &str) -> bool {
    let absolute = match normalized.strip_prefix("//") {
        Some(rest) => format!("http://{rest}"),
        None => normalized.to_string(),
    };
    let Ok(url) = Url::parse(&absolute) else {
        return false;
    };
    matches!(url.scheme(), "http" | "https") && url.host().is_some_and(is_private_host)
}
