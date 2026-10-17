//! Neutralisation du texte venant du courrier : contrôles simples, direction, accents.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use mira_tui::view::safe_text::{display_safe, display_safe_multiline};

/// La cloche et le retour chariot sont retirés.
#[test]
fn display_safe_strips_bell_and_carriage_return() {
    assert_eq!(display_safe("a\u{7}b\rc\u{8}d\u{0}e"), "abcde");
}

/// Les contrôles de direction du texte sont retirés.
#[test]
fn display_safe_strips_bidi_overrides() {
    assert_eq!(display_safe("a\u{202e}b\u{2066}c"), "abc");
}

/// Sur une ligne, les sauts de ligne et tabulations deviennent des espaces.
#[test]
fn display_safe_turns_newline_and_tab_into_spaces() {
    assert_eq!(display_safe("a\nb\tc"), "a b c");
}

/// Le texte multiligne garde les sauts de ligne mais retire le reste.
#[test]
fn display_safe_multiline_keeps_newlines_only() {
    assert_eq!(display_safe_multiline("a\r\nb\u{1b}[2J\u{7}c\n"), "a\nbc\n");
}

/// Le texte ordinaire, accents compris, reste intact.
#[test]
fn display_safe_keeps_plain_text() {
    assert_eq!(display_safe("Réunion à 14h — ok ✓"), "Réunion à 14h — ok ✓");
}
