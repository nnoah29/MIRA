#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Analyse du style : registre, ouvertures sans nom, longueurs, ponctuation, émojis, seuil d'exploitation.

use mira_core::ai::style::analyze;
use mira_vocabulary::StyleRegister;

/// `n` copies de ce mail
fn many(text: &str, n: usize) -> Vec<String> {
    vec![text.to_string(); n]
}

/// Des mails qui tutoient donnent le registre « tu »
#[test]
fn tutoiement_is_detected() {
    let profile = analyze(&many(
        "Salut Paul,\nTu peux me dire ce que tu en penses ? Ton avis compte.",
        6,
    ));
    assert_eq!(profile.register, StyleRegister::Tu);
    assert_eq!(profile.samples, 6);
    assert!(profile.usable());
}

/// Des mails qui vouvoient donnent le registre « vous »
#[test]
fn vouvoiement_is_detected() {
    let profile = analyze(&many(
        "Bonjour Madame,\nPouvez-vous me confirmer votre venue ? Merci.",
        6,
    ));
    assert_eq!(profile.register, StyleRegister::Vous);
}

/// Autant de mails avec tu que vous : registre mixte
#[test]
fn an_even_split_is_mixed() {
    let mut mails = many("Tu viens ? Ton avis ?", 3);
    mails.extend(many("Vous venez ? Votre avis ?", 3));
    assert_eq!(analyze(&mails).register, StyleRegister::Mixed);
}

/// Sans aucune marque de registre, il reste inconnu
#[test]
fn no_marker_leaves_the_register_unknown() {
    assert_eq!(analyze(&many("Ok.", 6)).register, StyleRegister::Unknown);
}
