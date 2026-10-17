#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! `mira provider list` : catalogue en JSON versionné et en texte.

#[allow(dead_code)]
mod cli_support;

use cli_support::run_cli;
use serde_json::Value;

/// La liste JSON contient les quatre providers, le repli en dernier
#[test]
fn json_list_has_four_providers_with_generic_last() {
    let result = run_cli(&["provider", "list", "--json"]);
    assert_eq!(result.code, 0);
    let document: Value = serde_json::from_str(&result.stdout).unwrap();
    assert_eq!(document["contract_version"]["major"], 1);
    assert_eq!(document["ok"], true);
    let ids: Vec<&str> = document["data"]["providers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|provider| provider["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["gmail", "outlook", "yahoo", "generic"]);
}

/// Le JSON d'un provider expose ses domaines et ses méthodes d'authentification
#[test]
fn json_list_exposes_domains_and_auth_methods() {
    let result = run_cli(&["provider", "list", "--json"]);
    let document: Value = serde_json::from_str(&result.stdout).unwrap();
    let gmail = &document["data"]["providers"][0];
    assert_eq!(gmail["domains"][0], "gmail.com");
    assert_eq!(gmail["auth_methods"][0]["type"], "oauth2");
    assert_eq!(gmail["auth_methods"][1]["type"], "app_password");
}

/// La sortie humaine liste une ligne par provider
#[test]
fn human_list_has_one_line_per_provider() {
    let result = run_cli(&["provider", "list"]);
    assert_eq!(result.code, 0);
    assert!(result.stderr.is_empty());
    let lines: Vec<&str> = result.stdout.lines().collect();
    assert_eq!(lines.len(), 4);
    assert!(lines[0].starts_with("gmail"));
    assert!(lines[3].starts_with("generic"));
    assert!(lines[3].contains("repli"));
}
