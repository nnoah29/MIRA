#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Rendu des variables d'un modèle : remplacement, variables visibles, une seule passe.

use mira_core::templates::render;
use mira_vocabulary::{ContactInfo, RenderContext};

/// Contexte complet : toutes les variables ont une valeur
fn full_context() -> RenderContext {
    RenderContext {
        contact: Some(ContactInfo {
            name: Some("Durand".to_string()),
            first_name: Some("Léa".to_string()),
            email: Some("lea@x.fr".to_string()),
        }),
        account_name: Some("Noah".to_string()),
        account_email: Some("n@x.fr".to_string()),
        subject: Some("Devis".to_string()),
        date: Some("25/10/2026".to_string()),
        time: Some("14:30".to_string()),
    }
}

/// Chaque variable du jeu est remplacée par sa valeur
#[test]
fn each_variable_is_replaced() {
    let text = "{{contact.nom}}|{{contact.prénom}}|{{contact.email}}|{{date}}|{{heure}}|{{moi.nom}}|{{moi.email}}|{{objet}}";
    let (rendered, unresolved) = render(text, &full_context());
    assert_eq!(
        rendered,
        "Durand|Léa|lea@x.fr|25/10/2026|14:30|Noah|n@x.fr|Devis"
    );
    assert!(unresolved.is_empty());
}

/// Les espaces à l'intérieur des accolades sont tolérés
#[test]
fn spaces_inside_braces_are_tolerated() {
    let (rendered, unresolved) = render("le {{ date }}", &full_context());
    assert_eq!(rendered, "le 25/10/2026");
    assert!(unresolved.is_empty());
}

/// Une variable inconnue reste visible et est signalée
#[test]
fn unknown_variable_stays_visible_and_is_listed() {
    let (rendered, unresolved) = render("a {{truc}} b", &full_context());
    assert_eq!(rendered, "a {{truc}} b");
    assert_eq!(unresolved, ["truc"]);
}

/// Les variables non résolues sont listées une fois, dans l'ordre de première apparition
#[test]
fn unresolved_is_deduplicated_in_order() {
    let (_, unresolved) = render("{{b}} {{a}} {{b}}", &full_context());
    assert_eq!(unresolved, ["b", "a"]);
}

/// Des accolades non fermées ne font pas échouer le rendu
#[test]
fn unclosed_braces_do_not_fail() {
    let (rendered, unresolved) = render("Bonjour {{contact.nom", &full_context());
    assert_eq!(rendered, "Bonjour {{contact.nom");
    assert!(unresolved.is_empty());
}
