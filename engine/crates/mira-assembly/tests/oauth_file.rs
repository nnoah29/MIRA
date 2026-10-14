#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Le fichier de configuration TOML fournit les identifiants d'application OAuth par provider.

use std::path::PathBuf;

use mira_assembly::mira_vocabulary::ProviderId;
use mira_assembly::oauth_file_load::load_oauth_config;
use mira_assembly::oauth_file_parse::parse_oauth_config;
use mira_assembly::oauth_file_path::oauth_config_path;

/// Deux providers déclarés donnent deux identifiants, espaces retirés
#[test]
fn parses_client_ids_per_provider() {
    let text = "[oauth.gmail]\nclient_id = \"g-id\"\n[oauth.outlook]\nclient_id = \"  o-id \"\n";
    let config = parse_oauth_config(text).unwrap();
    assert_eq!(config.client_ids[&ProviderId("gmail".into())], "g-id");
    assert_eq!(config.client_ids[&ProviderId("outlook".into())], "o-id");
    assert!(config.endpoint_overrides.is_empty());
}

/// Un texte vide ou sans section oauth donne une configuration vide
#[test]
fn empty_text_gives_empty_config() {
    assert!(parse_oauth_config("").unwrap().client_ids.is_empty());
}

/// Un identifiant vide, un champ inconnu ou un TOML cassé sont refusés
#[test]
fn rejects_empty_unknown_and_broken() {
    for text in [
        "[oauth.gmail]\nclient_id = \"  \"\n",
        "[oauth.gmail]\nclientid = \"x\"\n",
        "[oauth.gmail\n",
    ] {
        let error = parse_oauth_config(text).unwrap_err();
        assert_eq!(error.code, "invalid_input.parameter", "{text}");
    }
}

/// Un fichier absent donne une configuration vide, un fichier présent est lu
#[test]
fn missing_file_is_empty_and_present_file_is_read() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    assert!(load_oauth_config(&path).unwrap().client_ids.is_empty());
    std::fs::write(&path, "[oauth.gmail]\nclient_id = \"g-id\"\n").unwrap();
    assert_eq!(load_oauth_config(&path).unwrap().client_ids.len(), 1);
}

/// Un fichier invalide fait échouer le chargement avec le chemin dans l'erreur ou le message
#[test]
fn invalid_file_fails_to_load() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "n'importe quoi").unwrap();
    assert!(load_oauth_config(&path).is_err());
}

/// Priorité du chemin : MIRA_CONFIG, puis XDG_CONFIG_HOME absolu, puis ~/.config
#[test]
fn path_priority() {
    let explicit = Some(PathBuf::from("/x/c.toml"));
    let xdg = Some(PathBuf::from("/xdg"));
    let home = Some(PathBuf::from("/home/a"));
    assert_eq!(
        oauth_config_path(explicit, xdg.clone(), home.clone()),
        Some("/x/c.toml".into())
    );
    assert_eq!(
        oauth_config_path(None, xdg, home.clone()),
        Some("/xdg/mira/config.toml".into())
    );
    assert_eq!(
        oauth_config_path(None, Some("rel".into()), home),
        Some("/home/a/.config/mira/config.toml".into())
    );
    assert_eq!(oauth_config_path(None, None, None), None);
}
