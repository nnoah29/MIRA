//! HTML d'origine d'un message, lu par l'hôte du processus.

use crate::session::host;

pub use mira_host::original_html::html_of_raw;

/// Lit la source brute locale du message et rend l'enveloppe `{html}` (`null` sans partie HTML).
pub async fn fetch(message: &str) -> String {
    host().original_html(message).await
}
