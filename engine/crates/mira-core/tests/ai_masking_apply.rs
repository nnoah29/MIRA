#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Masqueur : repères numérotés par catégorie, valeurs répétées, repères déjà présents et mode coupé.

mod support_masking;

use mira_core::ai::masking::Masker;
use mira_vocabulary::{MaskingMode, MaskingSettings};
use support_masking::{ALL, mask_on, masker_all};

const IBAN: &str = "FR7630006000011234567890189";

/// Chaque catégorie compte à partir de 1, dans l'ordre du texte
#[test]
fn placeholders_are_numbered_per_category_from_one() {
    let mut masker = masker_all();
    let text = format!("{IBAN} puis 4111111111111111, puis GB82WEST12345698765432.");
    assert_eq!(
        masker.mask(&text),
        "[IBAN_1] puis [CARTE_1], puis [IBAN_2]."
    );
    let more = masker.mask("a@b.fr, mdp: secretX\ncode 1234, 06 12 34 56 78");
    assert_eq!(more, "[EMAIL_1], mdp: [SECRET_1]\ncode [CODE_1], [TEL_1]");
    assert_eq!(masker.mask("x c@d.fr"), "x [EMAIL_2]");
}

/// La même valeur écrite deux fois reçoit deux repères, tous deux restitués
#[test]
fn same_value_twice_gets_two_placeholders() {
    let mut masker = masker_all();
    let masked = masker.mask(&format!("{IBAN} et {IBAN}"));
    assert_eq!(masked, "[IBAN_1] et [IBAN_2]");
    let mut restorer = masker.into_restorer();
    assert_eq!(restorer.push(&masked), format!("{IBAN} et {IBAN}"));
}

/// Un repère déjà présent dans les textes fait sauter son numéro, y compris celui d'un autre message
#[test]
fn existing_placeholder_in_text_is_skipped() {
    let mut masker = masker_all();
    let first = masker.mask(&format!("voir [IBAN_1] puis {IBAN}"));
    assert_eq!(first, "voir [IBAN_1] puis [IBAN_2]");
    let mut other = masker_all();
    other.reserve("note : [IBAN_1] et [IBAN_2]");
    assert_eq!(other.mask(IBAN), "[IBAN_3]");
}

/// Mode coupé ou aucune catégorie : le texte est rendu tel quel et rien n'est retenu
#[test]
fn mask_off_returns_the_text_unchanged() {
    let off = MaskingSettings {
        mode: MaskingMode::Off,
        categories: ALL.to_vec(),
    };
    let mut masker = Masker::new(&off);
    let text = format!("{IBAN} a@b.fr 4111111111111111");
    assert_eq!(masker.mask(&text), text);
    assert_eq!(masker.count(), 0);
    let mut none = Masker::new(&mask_on(&[]));
    assert_eq!(none.mask(&text), text);
}

/// Le compte est le nombre de valeurs masquées, sur tous les textes de la requête
#[test]
fn count_is_the_number_of_masked_values() {
    let mut masker = masker_all();
    assert_eq!(masker.count(), 0);
    masker.mask("rien à signaler");
    assert_eq!(masker.count(), 0);
    masker.mask(&format!("{IBAN} {IBAN}"));
    masker.mask("a@b.fr");
    assert_eq!(masker.count(), 3);
    assert!(!format!("{masker:?}").contains(IBAN));
}
