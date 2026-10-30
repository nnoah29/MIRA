#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Masquage sur plusieurs tours : nouvelles entrées du masqueur, restituteur étendu, texte complet restitué.

mod support_masking;

use mira_core::ai::masking::Restorer;
use support_masking::{feed, masker_all};

const IBAN: &str = "FR7630006000011234567890189";
const CARD: &str = "4111111111111111";

/// `take_new` ne rend que les entrées ajoutées depuis son appel précédent
#[test]
fn masker_take_new_returns_only_new_entries() {
    let mut masker = masker_all();
    masker.mask(&format!("iban {IBAN}"));
    let first = masker.take_new();
    assert_eq!(first, vec![("[IBAN_1]".to_string(), IBAN.to_string())]);
    assert!(masker.take_new().is_empty());
    masker.mask(&format!("carte {CARD}"));
    let second = masker.take_new();
    assert_eq!(second, vec![("[CARTE_1]".to_string(), CARD.to_string())]);
}

/// Un restituteur étendu après sa création restitue les nouveaux repères, même coupés entre deux morceaux
#[test]
fn restorer_extend_restores_new_placeholders_across_chunks() {
    let mut masker = masker_all();
    masker.mask(&format!("iban {IBAN}"));
    let mut restorer = Restorer::new_empty();
    restorer.extend(masker.take_new());
    masker.mask(&format!("carte {CARD}"));
    restorer.extend(masker.take_new());
    let reply = "voici [IBAN_1] puis [CARTE_1] fin";
    for cut in 1..=reply.chars().count() {
        let mut copy = Restorer::new_empty();
        copy.extend(vec![
            ("[IBAN_1]".to_string(), IBAN.to_string()),
            ("[CARTE_1]".to_string(), CARD.to_string()),
        ]);
        let expected = format!("voici {IBAN} puis {CARD} fin");
        assert_eq!(feed(&mut copy, reply, cut), expected, "taille {cut}");
    }
}

/// `restore_text` restitue un texte complet, repères connus seulement, casse ignorée
#[test]
fn restore_text_restores_known_placeholders_only() {
    let mut restorer = Restorer::new_empty();
    restorer.extend(vec![("[IBAN_1]".to_string(), IBAN.to_string())]);
    let restored = restorer.restore_text("de [iban_1] ou [IBAN_2] ou [");
    assert_eq!(restored, format!("de {IBAN} ou [IBAN_2] ou ["));
}

/// `restore_text` ne touche pas à l'attente du flux en cours
#[test]
fn restore_text_leaves_stream_state_alone() {
    let mut restorer = Restorer::new_empty();
    restorer.extend(vec![("[IBAN_1]".to_string(), IBAN.to_string())]);
    assert_eq!(restorer.push("début [IBAN"), "début ");
    assert_eq!(restorer.restore_text("[IBAN_1]"), IBAN);
    assert_eq!(restorer.push("_1]"), IBAN);
}
