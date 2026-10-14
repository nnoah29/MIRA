//! Assainissement : suppression du script et des gestionnaires, blocage des ressources distantes.

use std::borrow::Cow;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use super::remote::{is_blocked_until_allowed, is_data_link};

/// HTML sûr à afficher, avec la liste des ressources distantes retirées.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafeHtml {
    pub html: String,
    pub blocked_resources: Vec<String>,
}

/// Assainit un HTML reçu ; les ressources distantes ne sont gardées que si `allow_remote`
pub fn sanitize_html(input: &str, allow_remote: bool) -> SafeHtml {
    let blocked = Arc::new(Mutex::new(Vec::new()));
    let html = builder(allow_remote, Arc::clone(&blocked))
        .clean(input)
        .to_string();
    let blocked_resources = blocked.lock().unwrap_or_else(|p| p.into_inner()).clone();
    SafeHtml {
        html,
        blocked_resources,
    }
}

/// Configure `ammonia` : schémas autorisés, liens neutralisés, filtre des attributs distants
fn builder(allow_remote: bool, blocked: Arc<Mutex<Vec<String>>>) -> ammonia::Builder<'static> {
    let mut builder = ammonia::Builder::default();
    builder
        .url_schemes(HashSet::from(["http", "https", "mailto", "cid", "data"]))
        .url_relative(ammonia::UrlRelative::Deny)
        .link_rel(Some("noopener noreferrer nofollow"))
        .add_tag_attributes("table", &["background"])
        .add_tag_attributes("td", &["background"])
        .add_tag_attributes("th", &["background"])
        .add_tag_attributes("tr", &["background"])
        .attribute_filter(move |element, attribute, value| {
            filter_attribute(element, attribute, value, allow_remote, &blocked)
        });
    builder
}

/// Retire un attribut qui chargerait une ressource distante non autorisée, et la note
fn filter_attribute<'v>(
    element: &str,
    attribute: &str,
    value: &'v str,
    allow_remote: bool,
    blocked: &Mutex<Vec<String>>,
) -> Option<Cow<'v, str>> {
    if is_data_link(attribute, value) {
        return None;
    }
    if allow_remote || !is_blocked_until_allowed(element, attribute, value) {
        return Some(Cow::Borrowed(value));
    }
    blocked
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .push(value.to_string());
    None
}
