//! Désabonnement annoncé par `List-Unsubscribe` et `List-Unsubscribe-Post` (11.6).

use mail_parser::Message;
use mira_vocabulary::Unsubscribe;

/// Éléments entre chevrons d'un en-tête `List-Unsubscribe`.
fn angle_items(text: &str) -> Vec<&str> {
    let mut items = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find('<') {
        let after = &rest[start + 1..];
        let Some(end) = after.find('>') else { break };
        items.push(after[..end].trim());
        rest = &after[end + 1..];
    }
    items
}

/// Premier élément qui commence par le préfixe (sans tenir compte de la casse).
fn first_with_prefix(items: &[&str], prefix: &str) -> Option<String> {
    items
        .iter()
        .find(|item| item.to_ascii_lowercase().starts_with(prefix))
        .map(|item| (*item).to_string())
}

/// Lit les moyens de désabonnement d'un message ; `None` si aucun n'est exploitable.
pub(crate) fn unsubscribe_from(message: &Message<'_>) -> Option<Unsubscribe> {
    let items = angle_items(message.header_raw("List-Unsubscribe")?);
    let https_url = first_with_prefix(&items, "https:");
    let mailto = first_with_prefix(&items, "mailto:");
    if https_url.is_none() && mailto.is_none() {
        return None;
    }
    let one_click = https_url.is_some()
        && message
            .header_raw("List-Unsubscribe-Post")
            .is_some_and(|post| {
                post.to_ascii_lowercase()
                    .contains("list-unsubscribe=one-click")
            });
    Some(Unsubscribe {
        https_url,
        mailto,
        one_click,
    })
}
