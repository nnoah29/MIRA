#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Le secret d'application est optionnel dans le fichier de configuration et ne fuit pas.

use mira_assembly::mira_vocabulary::ProviderId;
use mira_assembly::oauth_file_parse::parse_oauth_config;

/// Un client_secret déclaré est rangé (espaces retirés) ; sans client_secret, aucun secret n'est rangé
#[test]
fn secret_is_optional_and_trimmed() {
    let text = "[oauth.gmail]\nclient_id = \"g\"\nclient_secret = \" abc \"\n[oauth.outlook]\nclient_id = \"o\"\n";
    let config = parse_oauth_config(text).unwrap();
    let gmail = ProviderId("gmail".into());
    assert_eq!(config.client_secrets[&gmail].expose(), "abc");
    assert!(
        !config
            .client_secrets
            .contains_key(&ProviderId("outlook".into()))
    );
}

/// Un secret vide est refusé, et ni l'erreur ni le Debug de la configuration ne montrent le secret
#[test]
fn empty_secret_refused_and_value_never_printed() {
    let error =
        parse_oauth_config("[oauth.gmail]\nclient_id = \"g\"\nclient_secret = \"\"\n").unwrap_err();
    assert_eq!(error.code, "invalid_input.parameter");
    let typed = parse_oauth_config("[oauth.gmail]\nclient_id = \"g\"\nclient_secret = 12345678\n")
        .unwrap_err();
    assert!(!typed.message.contains("12345678"), "{}", typed.message);
    let config =
        parse_oauth_config("[oauth.gmail]\nclient_id = \"g\"\nclient_secret = \"topsecret\"\n")
            .unwrap();
    assert!(!format!("{config:?}").contains("topsecret"));
}
