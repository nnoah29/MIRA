//! Neutralisation du texte venant du courrier : séquences d'échappement complètes.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use mira_tui::view::safe_text::display_safe;

/// Une séquence d'effacement d'écran disparaît entièrement.
#[test]
fn display_safe_strips_ansi_clear_screen() {
    assert_eq!(display_safe("a\u{1b}[2Jb"), "ab");
}

/// Les séquences CSI avec paramètres et intermédiaires sont retirées en entier.
#[test]
fn display_safe_strips_csi_with_parameters() {
    assert_eq!(display_safe("x\u{1b}[38;2;1;2;3mY\u{1b}[?25lz"), "xYz");
}

/// Une chaîne OSC se termine par BEL ou par ST et disparaît avec son contenu.
#[test]
fn display_safe_strips_osc_with_bel_and_st() {
    assert_eq!(display_safe("a\u{1b}]0;titre\u{7}b"), "ab");
    assert_eq!(display_safe("a\u{1b}]52;c;AAAA\u{1b}\\b"), "ab");
}

/// Une chaîne OSC jamais terminée est retirée jusqu'à la fin du texte.
#[test]
fn display_safe_drops_unterminated_osc() {
    assert_eq!(display_safe("a\u{1b}]0;titre sans fin"), "a");
}

/// Les introducteurs 8 bits (C1) sont traités comme leurs équivalents ESC.
#[test]
fn display_safe_strips_c1_introducers() {
    assert_eq!(display_safe("a\u{9b}2Jb\u{9d}0;t\u{7}c"), "abc");
}

/// Une séquence ESC tronquée ou un ESC isolé ne laisse aucun caractère de contrôle.
#[test]
fn display_safe_handles_truncated_escapes() {
    assert_eq!(display_safe("a\u{1b}"), "a");
    assert_eq!(display_safe("a\u{1b}[31"), "a");
    assert_eq!(display_safe("a\u{1b}(Bb"), "ab");
}
