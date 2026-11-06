//! Une seule ligne `error` quand la demande est refusée avant tout envoi.

use mira_assembly::api::AiRequest;
use mira_assembly::mira_vocabulary::{AiConnectionId, AiProviderId};

use super::chat_fixture::chat_json;
use super::chat_start::{ask_ai, collect_lines, start};
use super::driver::Driver;

/// Une connexion inconnue donne une seule ligne d'erreur, avec son code stable.
pub async fn local_error_is_a_single_error_line<D: Driver>() {
    let f = start::<D>().await;
    let lines = collect_lines(&f.driver, &chat_json(Some("inconnue"), "Salut")).await;
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert_eq!(lines[0]["type"], "error");
    assert_eq!(lines[0]["error"]["code"], "not_found.ai_connection");
    assert!(lines[0]["error"]["message"].as_str().is_some());
    assert!(f.server.requests().is_empty());
    assert!(f.driver.close().await.contains("\"ok\":true"));
}

/// Une destination distante sans consentement est refusée localement, sans requête émise.
pub async fn consent_required_is_an_error_line_with_code<D: Driver>() {
    let f = start::<D>().await;
    let provider = AiProviderId("generic".to_string());
    let base_url = Some("https://ai.provider.test/v1".to_string());
    let add = AiRequest::AddAiConnection {
        provider,
        label: "Distante".into(),
        base_url,
    };
    let created = ask_ai(&f.driver, add).await;
    let id = created["id"].as_str().unwrap();
    assert_ne!(
        AiConnectionId::from_raw(id),
        AiConnectionId::from_raw(f.connection.clone())
    );
    let lines = collect_lines(&f.driver, &chat_json(Some(id), "Salut")).await;
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert_eq!(lines[0]["type"], "error");
    assert_eq!(
        lines[0]["error"]["code"],
        "incompatible_state.ai_consent_required"
    );
    assert!(f.driver.close().await.contains("\"ok\":true"));
}

/// Une demande illisible donne une seule ligne d'erreur qui ne reprend pas son texte.
pub async fn invalid_request_json_is_an_error_line<D: Driver>() {
    let f = start::<D>().await;
    let lines = collect_lines(&f.driver, "{ceci n'est pas du JSON SECRET-TEXTE").await;
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert_eq!(lines[0]["type"], "error");
    assert!(!lines[0].to_string().contains("SECRET-TEXTE"));
    assert!(f.driver.close().await.contains("\"ok\":true"));
}

/// Sans moteur ouvert, une seule ligne d'erreur.
pub async fn no_engine_open_is_an_error_line<D: Driver>() {
    let driver = D::start(None).await;
    let lines = collect_lines(&driver, &chat_json(Some("c"), "Salut")).await;
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert_eq!(lines[0]["type"], "error");
    assert_eq!(lines[0]["error"]["message"], "moteur non ouvert");
}
