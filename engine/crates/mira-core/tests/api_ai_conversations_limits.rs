#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Contrat 1.4 : bornes du titre et du contenu d'une conversation d'IA.

mod support_ai_conversations;
mod support_api;

use support_ai_conversations::{list, save, start};

/// Objet JSON dont le texte fait exactement `bytes` octets
fn object_of_size(bytes: usize) -> String {
    let filler = "a".repeat(bytes - "{\"k\":\"\"}".len());
    format!("{{\"k\":\"{filler}\"}}")
}

/// Un titre vide ou blanc est refusé
#[tokio::test]
async fn empty_or_blank_title_is_refused() {
    let f = start();
    for title in ["", "   ", "\n\t"] {
        let refused = save(&f.engine, "c-1", title, "{}").await;
        assert_eq!(refused.unwrap_err(), "invalid_input.ai_conversation_title");
    }
    assert!(list(&f.engine, 50, None).await.unwrap().is_empty());
}

/// Le titre se compte en caractères : 80 passent, 81 non, accents et emoji compris
#[tokio::test]
async fn title_counts_characters_not_bytes() {
    let f = start();
    for unit in ["é", "😀", "a"] {
        let ok = save(&f.engine, "c-1", &unit.repeat(80), "{}").await;
        assert_eq!(ok.unwrap().title.chars().count(), 80);
        let refused = save(&f.engine, "c-1", &unit.repeat(81), "{}").await;
        assert_eq!(refused.unwrap_err(), "invalid_input.ai_conversation_title");
    }
}

/// 2 Mio exactement passent, un octet de plus est refusé
#[tokio::test]
async fn payload_limit_is_exactly_2_mib() {
    let f = start();
    let limit = 2 * 1024 * 1024;
    save(&f.engine, "c-1", "T", &object_of_size(limit))
        .await
        .unwrap();
    let refused = save(&f.engine, "c-2", "T", &object_of_size(limit + 1)).await;
    assert_eq!(
        refused.unwrap_err(),
        "invalid_input.ai_conversation_too_large"
    );
    assert_eq!(list(&f.engine, 50, None).await.unwrap().len(), 1);
}

/// Un contenu qui n'est pas un objet JSON est refusé
#[tokio::test]
async fn payload_must_be_a_json_object() {
    let f = start();
    for payload in ["[]", "42", "\"texte\"", "pas du json", ""] {
        let refused = save(&f.engine, "c-1", "T", payload).await;
        assert_eq!(
            refused.unwrap_err(),
            "invalid_input.ai_conversation_payload",
            "{payload}"
        );
    }
}

/// Un refus ne répète ni le titre ni le contenu dans son message
#[tokio::test]
async fn refusals_do_not_echo_the_content() {
    let f = start();
    let request = mira_core::api::AiRequest::SaveAiConversation {
        id: "c-1".into(),
        title: "TITRE-PRIVE ".repeat(20),
        payload_version: 1,
        payload: "{}".into(),
    };
    let error = f
        .engine
        .execute(mira_core::api::Request::Ai(request))
        .await
        .unwrap_err();
    assert!(!format!("{error:?}").contains("TITRE-PRIVE"), "{error:?}");
}
