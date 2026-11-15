#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Nettoyage d'un mail envoyé avant l'analyse du style : citations, signature et lignes vides disparaissent.

use mira_core::ai::style::clean_mail;

/// Les lignes citées par `>` ne sont pas du texte de l'utilisateur
#[test]
fn quoted_lines_are_removed() {
    let cleaned = clean_mail("Merci pour ton retour.\n> Le message d'origine\n> suite\nÀ bientôt");
    assert_eq!(cleaned, "Merci pour ton retour.\nÀ bientôt");
}

/// Une attaque de citation française ou anglaise coupe tout ce qui suit
#[test]
fn a_reply_header_cuts_the_rest() {
    let fr =
        clean_mail("Ok pour moi.\n\nLe lun. 3 mars 2025 à 10:00, Paul a écrit :\nancien texte");
    assert_eq!(fr, "Ok pour moi.");
    let en = clean_mail("Sounds good.\nOn Mon, Mar 3, 2025 at 10:00 AM Paul wrote:\nold text");
    assert_eq!(en, "Sounds good.");
}

/// Tout ce qui suit la ligne `-- ` est une signature
#[test]
fn the_signature_is_cut() {
    assert_eq!(
        clean_mail("Bonjour,\nOui.\n-- \nNoah\n06 00 00 00 00"),
        "Bonjour,\nOui."
    );
}

/// Les lignes vides en excès sont ramenées à rien et le texte est rogné
#[test]
fn blank_lines_and_edges_are_trimmed() {
    assert_eq!(clean_mail("\n\n  Salut  \n\n\nOui\n\n"), "Salut\nOui");
}
