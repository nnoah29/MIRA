#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Invariant 6 : toute opération du contrat a au moins une commande `mira <domaine> <verbe>`.

#[allow(dead_code)]
mod cli_support;
#[allow(dead_code)]
mod request_catalog;

use std::collections::BTreeSet;

use cli_support::run_cli;
use request_catalog::examples::all_examples;
use request_catalog::lines_read_write::READ_WRITE_LINES;
use request_catalog::{request_of_line, serialized_name, variant_name};

/// Table des commandes : au moins une ligne par variante de `Request`.
const COMMAND_LINES: &[&[&str]] = &[
    &["engine", "info"],
    &["engine", "status"],
    &["provider", "list"],
    &["provider", "detect", "a@b.c"],
    &["account", "prepare", "--address", "a@b.c"],
    &["account", "test", "--address", "a@b.c", "--password-stdin"],
    &["account", "add", "--address", "a@b.c", "--password-stdin"],
    &["account", "update", "acc", "--display-name", "N"],
    &["account", "pause", "acc"],
    &["account", "resume", "acc"],
    &["account", "delete", "acc"],
    &["account", "list"],
    &["identity", "list", "acc"],
    &[
        "identity",
        "add",
        "acc",
        "--address",
        "a@b.c",
        "--display-name",
        "N",
    ],
    &["identity", "update", "ident", "--signature", "S"],
    &["identity", "set-default", "ident"],
    &["auth", "oauth-start", "a@b.c"],
    &["auth", "oauth-complete", "flow"],
    &["auth", "oauth-cancel", "flow"],
    &["auth", "app-password", "acc", "--password-stdin"],
    &["auth", "reauthenticate", "acc"],
    &["operation", "get", "op"],
    &["operation", "cancel", "op"],
    &["events", "--after", "3", "--limit", "5"],
];

/// Invariant 6 : chaque variante de `Request` est produite par une ligne de commande
#[test]
fn invariant_6_every_request_has_a_cli_command() {
    let produced: BTreeSet<String> = COMMAND_LINES
        .iter()
        .chain(READ_WRITE_LINES)
        .map(|line| serialized_name(&request_of_line(line)))
        .collect();
    for example in all_examples() {
        assert_eq!(variant_name(&example), serialized_name(&example));
        assert!(
            produced.contains(&serialized_name(&example)),
            "aucune commande pour {}",
            serialized_name(&example)
        );
    }
}

/// Aucune commande ne produit deux fois exactement la même ligne de table
#[test]
fn command_table_has_no_duplicate_line() {
    let all: Vec<&&[&str]> = COMMAND_LINES.iter().chain(READ_WRITE_LINES).collect();
    let unique: BTreeSet<&&&[&str]> = all.iter().collect();
    assert_eq!(unique.len(), all.len());
}

/// Un mot de passe ne se passe pas en argument : l'option n'existe pas
#[test]
fn password_value_flag_does_not_exist() {
    let result = run_cli(&["account", "add", "--address", "a@b.c", "--password", "x"]);
    assert_eq!(result.code, 10);
    assert!(result.stdout.is_empty());
}

/// Le secret lu sur stdin se retrouve dans la requête mais jamais dans son affichage de débogage
#[test]
fn stdin_secret_is_masked_in_debug() {
    let line = ["auth", "app-password", "acc", "--password-stdin"];
    let request = request_of_line(&line);
    let debug = format!("{request:?}");
    assert!(!debug.contains("un-secret"));
    assert!(debug.contains("***"));
}
