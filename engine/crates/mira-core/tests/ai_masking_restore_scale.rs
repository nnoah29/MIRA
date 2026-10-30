#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Restitution : coût borné avec une grande table et sortie identique morceau par morceau.

mod support_masking;

use std::time::{Duration, Instant};

use mira_core::ai::masking::Masker;
use mira_vocabulary::SensitiveCategory::Email;
use support_masking::{mask_on, restorer_for};

/// Une grande table et des milliers de crochets dans un seul morceau restent rapides
#[test]
fn large_table_with_many_brackets_stays_fast() {
    let mut masker = Masker::new(&mask_on(&[Email]));
    let addresses: Vec<String> = (0..2_000).map(|i| format!("user{i}@example.org")).collect();
    masker.mask(&addresses.join(" "));
    let mut restorer = masker.into_restorer();
    let started = Instant::now();
    let out = restorer.push(&"[".repeat(100_000));
    let held = restorer.pending_len();
    let tail = restorer.push("[EMAIL_2000] [email_1]");
    assert!(started.elapsed() < Duration::from_secs(2), "trop lent");
    assert_eq!(out.len() + held, 100_000);
    assert_eq!(tail, "[user1999@example.org user0@example.org");
}

/// Les sorties de chaque appel à `push` sont celles de la version précédente, casse et crochets inconnus compris
#[test]
fn restore_output_is_identical_to_before_on_split_inputs() {
    let (_, mut restorer) = restorer_for("FR7630006000011234567890189 4111111111111111");
    let chunks = [
        "a [IB",
        "AN_1] b [ca",
        "rte_1] [IBAN_",
        "2] [[",
        "IBAN_1",
        "] fin [",
    ];
    let pushed: Vec<String> = chunks.iter().map(|chunk| restorer.push(chunk)).collect();
    assert_eq!(
        pushed,
        [
            "a ",
            "FR7630006000011234567890189 b ",
            "4111111111111111 ",
            "[IBAN_2] [",
            "",
            "FR7630006000011234567890189 fin "
        ]
    );
    assert_eq!(restorer.finish(), "[");
}
