#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! `mira provider detect` : sortie JSON versionnée, sortie humaine, erreurs.

#[allow(dead_code)]
mod cli_support;

use cli_support::run_cli;
use serde_json::Value;

/// Analyse la sortie standard comme un document JSON
fn parse(stdout: &str) -> Value {
    serde_json::from_str(stdout).unwrap()
}

/// Sortie JSON versionnée pour une adresse reconnue
#[test]
fn scenario_cli_provider_detect_json() {
    let result = run_cli(&["provider", "detect", "Moi@GMAIL.com", "--json"]);
    assert_eq!(result.code, 0);
    assert!(result.stderr.is_empty());
    let document = parse(&result.stdout);
    assert_eq!(document["contract_version"]["major"], 1);
    assert_eq!(document["contract_version"]["minor"], 0);
    assert_eq!(document["ok"], true);
    assert_eq!(document["data"]["address"], "Moi@gmail.com");
    assert_eq!(document["data"]["provider"]["id"], "gmail");
    assert_eq!(
        document["data"]["servers"]["imap"]["host"],
        "imap.gmail.com"
    );
    assert_eq!(document["data"]["servers"]["smtp"]["port"], 465);
    assert_eq!(document["data"]["servers"]["smtp"]["security"], "tls");
}

/// Une adresse inconnue est servie par le repli générique
#[test]
fn unknown_address_is_detected_as_generic() {
    let result = run_cli(&["--json", "provider", "detect", "x@inconnu.org"]);
    assert_eq!(result.code, 0);
    let document = parse(&result.stdout);
    assert_eq!(document["data"]["provider"]["id"], "generic");
    assert_eq!(
        document["data"]["servers"]["imap"]["host"],
        "imap.inconnu.org"
    );
}

/// La sortie humaine nomme le provider et les serveurs
#[test]
fn human_output_names_provider_and_servers() {
    let result = run_cli(&["provider", "detect", "x@hotmail.fr"]);
    assert_eq!(result.code, 0);
    assert!(result.stdout.contains("outlook"));
    assert!(result.stdout.contains("outlook.office365.com:993 (tls)"));
    assert!(result.stdout.contains("smtp.office365.com:587 (starttls)"));
    assert!(serde_json::from_str::<Value>(&result.stdout).is_err());
}

/// Adresse invalide en JSON : enveloppe d'erreur sur stdout, code 10
#[test]
fn invalid_address_gives_exit_code_10_with_json_error() {
    let result = run_cli(&["provider", "detect", "pas-une-adresse", "--json"]);
    assert_eq!(result.code, 10);
    assert!(result.stderr.is_empty());
    let document = parse(&result.stdout);
    assert_eq!(document["contract_version"]["major"], 1);
    assert_eq!(document["ok"], false);
    assert_eq!(document["error"]["category"], "invalid_input");
    assert_eq!(document["error"]["code"], "invalid_input.address");
    assert!(document.get("data").is_none());
}

/// Adresse invalide en sortie humaine : message sur stderr, stdout vide, code 10
#[test]
fn invalid_address_human_output_goes_to_stderr() {
    let result = run_cli(&["provider", "detect", "pas-une-adresse"]);
    assert_eq!(result.code, 10);
    assert!(result.stdout.is_empty());
    assert!(result.stderr.contains("invalid_input.address"));
}
