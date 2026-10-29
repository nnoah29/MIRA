#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Restitution en flux : repères entiers ou coupés partout, inconnus, crochets isolés et attente bornée.

mod support_masking;

use support_masking::{feed, restorer_for};

const IBAN: &str = "FR7630006000011234567890189";

/// Un repère reçu d'un seul morceau redevient la valeur, le texte autour est conservé
#[test]
fn placeholder_is_restored_in_one_chunk() {
    let (masked, mut restorer) = restorer_for(&format!("mon iban {IBAN}"));
    assert_eq!(masked, "mon iban [IBAN_1]");
    assert_eq!(
        restorer.push("Votre IBAN [IBAN_1] est valide.\n"),
        format!("Votre IBAN {IBAN} est valide.\n")
    );
    assert_eq!(restorer.finish(), "");
}

/// Quelle que soit la coupure, y compris à chaque position, la sortie est la même
#[test]
fn placeholder_split_at_every_position_is_restored() {
    let reply = "é [IBAN_1] et [CARTE_1], [IBAN_2] [IBAN_ [[CARTE_1] ✓ [";
    let expected = format!("é {IBAN} et 4111111111111111, [IBAN_2] [IBAN_ [4111111111111111 ✓ [");
    for cut in 1..=reply.chars().count() {
        let (_, mut restorer) = restorer_for(&format!("{IBAN} 4111111111111111"));
        assert_eq!(feed(&mut restorer, reply, cut), expected, "taille {cut}");
    }
    for split in (0..=reply.len()).filter(|i| reply.is_char_boundary(*i)) {
        let (_, mut restorer) = restorer_for(&format!("{IBAN} 4111111111111111"));
        let mut out = restorer.push(&reply[..split]);
        out.push_str(&restorer.push(&reply[split..]));
        out.push_str(&restorer.finish());
        assert_eq!(out, expected, "coupure {split}");
    }
}

/// Un repère que le modèle invente, ou qui n'a pas été créé pour cette requête, n'est pas touché
#[test]
fn unknown_placeholder_passes_through() {
    let (_, mut restorer) = restorer_for(IBAN);
    let reply = "[IBAN_7] [SECRET_1] [IBAN_1x] [INCONNU] [IBAN_1]";
    let out = feed(&mut restorer, reply, 3);
    assert_eq!(
        out,
        format!("[IBAN_7] [SECRET_1] [IBAN_1x] [INCONNU] {IBAN}")
    );
}

/// Un crochet resté seul en fin de flux sort au `finish`
#[test]
fn lone_bracket_is_emitted_at_finish() {
    let (_, mut restorer) = restorer_for(IBAN);
    assert_eq!(restorer.push("fin [IBAN_"), "fin ");
    assert_eq!(restorer.pending_len(), "[IBAN_".len());
    assert_eq!(restorer.finish(), "[IBAN_");
    assert_eq!(restorer.finish(), "");
}

/// Sans crochet, rien n'est retenu, même avec une table pleine ou vide
#[test]
fn text_without_brackets_is_emitted_immediately() {
    let (_, mut restorer) = restorer_for(IBAN);
    assert_eq!(restorer.push("Bon"), "Bon");
    assert_eq!(restorer.push("jour é漢\n"), "jour é漢\n");
    assert_eq!(restorer.pending_len(), 0);
    let (_, mut empty) = restorer_for("rien");
    assert_eq!(empty.push("[IBAN_1] [x"), "[IBAN_1] [x");
    assert_eq!(empty.pending_len(), 0);
}

/// L'attente ne dépasse jamais la longueur du plus long repère connu
#[test]
fn pending_never_exceeds_the_longest_placeholder() {
    let (_, mut restorer) = restorer_for(&format!("{IBAN} 4111111111111111"));
    let longest = "[CARTE_1]".len();
    let hostile = "[[[IBAN_1[CARTE_[CARTE_1[IBAN_1] [CARTE_1]]]";
    for character in hostile.chars() {
        restorer.push(&character.to_string());
        assert!(restorer.pending_len() < longest);
    }
    assert!(restorer.pending_len() < longest);
}
