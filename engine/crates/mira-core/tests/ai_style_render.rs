#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Consigne de style ajoutée à la rédaction : courte, sans donnée personnelle, absente si le profil n'est pas exploitable.

use mira_core::ai::style::{analyze, render_style_notice};

/// Profil exploitable d'un utilisateur qui tutoie, ouvre par « Salut » et aime les émojis
fn lively() -> mira_vocabulary::StyleProfile {
    analyze(&vec![
        "Salut Paul,\nTu viens ? Ton avis ! 😀".to_string();
        6
    ])
}

/// Un profil non exploitable ne produit aucune consigne
#[test]
fn unusable_profile_renders_nothing() {
    assert_eq!(render_style_notice(&analyze(&[]), "notes"), None);
}

/// La consigne dit le registre, l'ouverture, la longueur et les émojis, sans le nom du destinataire
#[test]
fn notice_describes_register_greeting_length_and_emojis() {
    let notice = render_style_notice(&lively(), "").unwrap();
    assert!(notice.contains("tutoie"), "{notice}");
    assert!(notice.contains("Salut"), "{notice}");
    assert!(notice.contains("mots"), "{notice}");
    assert!(notice.to_lowercase().contains("émoji"), "{notice}");
    assert!(!notice.contains("Paul"));
}

/// Les notes de l'utilisateur sont reprises, coupées pour que la consigne reste courte
#[test]
fn notes_are_included_and_capped() {
    let notice = render_style_notice(&lively(), "Je signe toujours par un proverbe.").unwrap();
    assert!(notice.contains("proverbe"));
    let long = "x".repeat(2_000);
    let capped = render_style_notice(&lively(), &long).unwrap();
    assert!(capped.chars().count() <= 800, "{}", capped.chars().count());
}
