#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Textes des notifications : français et anglais, selon la langue du système.

use mira_host::notifier::Texts;

/// Une langue française donne des textes français.
#[test]
fn a_french_locale_gives_french_texts() {
    let texts = Texts::for_locale("fr_FR.UTF-8");
    assert_eq!(texts.new_message, "Nouveau message");
    assert_eq!(texts.many(4), "4 nouveaux messages");
}

/// Toute autre langue, ou une langue vide, donne des textes anglais.
#[test]
fn any_other_locale_gives_english_texts() {
    for locale in ["en_US.UTF-8", "de_DE", "", "C"] {
        let texts = Texts::for_locale(locale);
        assert_eq!(texts.new_message, "New message", "{locale}");
        assert_eq!(texts.many(2), "2 new messages", "{locale}");
    }
}

/// Les textes d'attente du trousseau existent dans les deux langues.
#[test]
fn the_keyring_texts_follow_the_locale() {
    let french = Texts::for_locale("fr");
    assert_eq!(
        french.keyring_locked_title,
        "MIRA attend le déverrouillage du trousseau"
    );
    let english = Texts::for_locale("en");
    assert_eq!(
        english.keyring_locked_title,
        "MIRA is waiting for your keyring"
    );
    assert!(!french.keyring_locked_body.is_empty() && !english.keyring_locked_body.is_empty());
}
