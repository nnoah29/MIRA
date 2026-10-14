#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Codes de sortie par catégorie d'erreur, et erreurs d'usage.

#[allow(dead_code)]
mod cli_support;

use cli_support::run_cli;
use mira_assembly::mira_vocabulary::ErrorCategory;
use mira_cli::exit_codes::exit_code_for;
use serde_json::Value;

/// Table complète catégorie vers code de sortie
#[test]
fn every_category_has_its_exit_code() {
    let table = [
        (ErrorCategory::InvalidInput, 10),
        (ErrorCategory::NotFound, 11),
        (ErrorCategory::IncompatibleState, 12),
        (ErrorCategory::Network, 13),
        (ErrorCategory::Tls, 14),
        (ErrorCategory::Authentication, 15),
        (ErrorCategory::Server, 16),
        (ErrorCategory::LocalStorage, 17),
        (ErrorCategory::Secrets, 18),
        (ErrorCategory::Cancelled, 19),
        (ErrorCategory::Internal, 84),
    ];
    for (category, code) in table {
        assert_eq!(exit_code_for(category), code, "{category:?}");
    }
}

/// Une sous-commande inconnue est une entrée invalide
#[test]
fn unknown_subcommand_is_invalid_input() {
    let result = run_cli(&["inconnue"]);
    assert_eq!(result.code, 10);
    assert!(result.stdout.is_empty());
    assert!(!result.stderr.is_empty());
}

/// Un argument manquant est une entrée invalide, enveloppée en JSON si demandé
#[test]
fn missing_argument_is_invalid_input_in_json() {
    let result = run_cli(&["provider", "detect", "--json"]);
    assert_eq!(result.code, 10);
    let document: Value = serde_json::from_str(&result.stdout).unwrap();
    assert_eq!(document["ok"], false);
    assert_eq!(document["error"]["category"], "invalid_input");
}

/// L'aide sort en 0 sur la sortie standard
#[test]
fn help_exits_zero_on_stdout() {
    let result = run_cli(&["--help"]);
    assert_eq!(result.code, 0);
    assert!(result.stdout.contains("provider"));
    assert!(result.stderr.is_empty());
}

/// `run` écrit sur les flux injectés, pas sur les flux du processus
#[test]
fn run_writes_only_to_injected_streams() {
    let result = run_cli(&["provider", "list"]);
    assert_eq!(result.code, 0);
    assert!(!result.stdout.is_empty());
    assert!(result.stderr.is_empty());
}
