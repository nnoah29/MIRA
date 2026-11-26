#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Enveloppe d'export YAML : une imbrication extrême est refusée sans épuiser la pile.

mod support_portability;

use mira_core::portability::envelope::unwrap;
use support_portability::envelope::code_of;

/// Dans un thread à petite pile, des milliers de niveaux d'imbrication échouent proprement
#[test]
fn deeply_nested_yaml_is_refused_without_stack_overflow() {
    let depth = 100_000;
    let text = format!("{}{}", "[".repeat(depth), "]".repeat(depth));
    let outcome = std::thread::Builder::new()
        .stack_size(512 * 1024)
        .spawn(move || code_of(unwrap(&text, "rules", "rules")))
        .unwrap()
        .join();
    let code = outcome.expect("le thread ne doit pas dépasser sa pile");
    assert_eq!(code, "invalid_input.yaml_syntax");
}

/// Même chose avec des objets imbriqués
#[test]
fn deeply_nested_mappings_are_refused_without_stack_overflow() {
    let depth = 50_000;
    let text = format!("{}1{}", "{a: ".repeat(depth), "}".repeat(depth));
    let outcome = std::thread::Builder::new()
        .stack_size(512 * 1024)
        .spawn(move || code_of(unwrap(&text, "rules", "rules")))
        .unwrap()
        .join();
    let code = outcome.expect("le thread ne doit pas dépasser sa pile");
    assert_eq!(code, "invalid_input.yaml_syntax");
}
