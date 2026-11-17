#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Mesures du style : ouvertures sans nom, longueurs médianes, exclamations, émojis, seuil d'exploitation.

use mira_core::ai::style::analyze;

/// `n` copies de ce mail
fn many(text: &str, n: usize) -> Vec<String> {
    vec![text.to_string(); n]
}

/// Les ouvertures sont comptées sans le nom qui suit, la plus fréquente d'abord, trois au plus
#[test]
fn greetings_never_keep_a_name() {
    let mut mails = many("Bonjour Marie,\nOui.", 3);
    mails.extend(many("Salut Paul,\nOui.", 2));
    mails.extend(many("Hello Jean,\nOui.", 1));
    mails.extend(many("Coucou Léa,\nOui.", 1));
    let profile = analyze(&mails);
    assert_eq!(profile.greetings, ["Bonjour", "Salut", "Hello"]);
    assert!(profile.greetings.iter().all(|g| !g.contains("Marie")));
}

/// Les longueurs médianes sont en mots, pour le mail et pour la phrase
#[test]
fn medians_are_counted_in_words() {
    let profile = analyze(&many("Un deux trois. Quatre cinq.", 5));
    assert_eq!(profile.median_words, 5);
    assert_eq!(profile.median_sentence_words, 2);
}

/// Les points d'exclamation et les émojis sont relevés
#[test]
fn exclamations_and_emojis_are_measured() {
    let calm = analyze(&many("Merci. Bonne journée.", 5));
    assert_eq!((calm.exclamation_permille, calm.emoji), (0, false));
    let lively = analyze(&many("Super ! Merci beaucoup ! 😀", 5));
    assert!(lively.exclamation_permille >= 500);
    assert!(lively.emoji);
}

/// Un mail fait de citations seules ne compte pas ; sous cinq mails le profil n'est pas exploitable
#[test]
fn too_few_usable_mails_is_not_usable() {
    let mut mails = many("> tout est cité", 4);
    mails.extend(many("Oui.", 4));
    let profile = analyze(&mails);
    assert_eq!(profile.samples, 4);
    assert!(!profile.usable());
    assert!(!analyze(&[]).usable());
}
