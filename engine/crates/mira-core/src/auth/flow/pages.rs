//! Pages montrées dans le navigateur quand l'autorisation se termine, sans aucune valeur issue de la requête.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use sha2::{Digest, Sha256};

const TEMPLATE: &str = include_str!("redirect_page.html");

/// Page affichée quand le code d'autorisation a bien été reçu
pub(super) fn success_page() -> String {
    render(
        "Autorisation reçue",
        "Vous pouvez fermer cet onglet et revenir à MIRA.",
        "Authorisation received. You can close this tab and go back to MIRA.",
    )
}

/// Page affichée quand l'accès a été refusé ou n'a pas abouti
pub(super) fn failure_page() -> String {
    render(
        "Autorisation non aboutie",
        "L'accès n'a pas été accordé. Revenez à MIRA pour réessayer.",
        "Access was not granted. Go back to MIRA to try again.",
    )
}

/// Remplit le modèle avec des textes fixes ; rien n'est repris de la requête du navigateur
fn render(title: &str, french: &str, english: &str) -> String {
    TEMPLATE
        .replace("{{title}}", title)
        .replace("{{fr}}", french)
        .replace("{{en}}", english)
}

/// Politique de sécurité de la réponse : rien n'est chargé, seul le style du modèle (par son empreinte) et la police intégrée passent
pub(super) fn policy() -> String {
    let style = TEMPLATE
        .split_once("<style>")
        .and_then(|(_, rest)| rest.split_once("</style>"))
        .map_or("", |(style, _)| style);
    let hash = STANDARD.encode(Sha256::digest(style.as_bytes()));
    format!("default-src 'none'; style-src 'sha256-{hash}'; font-src data:")
}
