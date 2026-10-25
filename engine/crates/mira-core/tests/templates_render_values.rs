#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Rendu des variables d'un modèle : valeurs vides ou absentes, une seule passe.

use mira_core::templates::render;
use mira_vocabulary::{ContactInfo, RenderContext};

/// Une valeur absente ou vide laisse la variable visible
#[test]
fn missing_or_empty_value_stays_visible() {
    let context = RenderContext {
        account_name: Some(String::new()),
        ..RenderContext::default()
    };
    let (rendered, unresolved) = render("{{contact.nom}} / {{moi.nom}}", &context);
    assert_eq!(rendered, "{{contact.nom}} / {{moi.nom}}");
    assert_eq!(unresolved, ["contact.nom", "moi.nom"]);
}

/// Une valeur qui contient une variable n'est jamais re-substituée
#[test]
fn value_is_not_resubstituted() {
    let context = RenderContext {
        contact: Some(ContactInfo {
            name: Some("{{date}}".to_string()),
            ..ContactInfo::default()
        }),
        date: Some("25/10/2026".to_string()),
        ..RenderContext::default()
    };
    let (rendered, unresolved) = render("Bonjour {{contact.nom}}", &context);
    assert_eq!(rendered, "Bonjour {{date}}");
    assert!(unresolved.is_empty());
}
