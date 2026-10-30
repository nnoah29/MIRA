#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Domaine « Ai » par la façade : clés d'API rangées au trousseau, validées, jamais renvoyées.

mod ai_models_support;
mod support_ai_api;
mod support_api;

use mira_core::api::AiRequest;
use mira_core::ports::SecretStore;
use mira_vocabulary::{AiSecretRef, ProfileId};
use support_ai_api::calls::{add, call_connection, list};
use support_ai_api::operations::set_key;
use support_ai_api::providers::{EDITABLE, started};
use support_ai_api::session::{queue_one_model, run_session};

const KEY: &str = "sk-live-TOP-SECRET-0123";

/// La clé rangée ne revient dans aucune réponse : seul `has_key` change
#[tokio::test]
async fn set_key_sets_has_key_and_never_returns_it() {
    let f = started().await;
    let created = add(&f.engine, EDITABLE, "A", Some(&f.server.base_url()))
        .await
        .unwrap();
    let after = set_key(&f.engine, &created.id, KEY).await.unwrap();
    assert!(after.has_key);
    assert!(list(&f.engine).await[0].has_key);
    assert!(!serde_json::to_string(&after).unwrap().contains(KEY));
    let location = AiSecretRef {
        profile: ProfileId::from_raw("default"),
        connection: created.id.clone(),
    };
    assert_eq!(f.secrets.get(&location).unwrap().unwrap().expose(), KEY);
}

/// Effacer la clé la retire du trousseau et remet `has_key` à faux
#[tokio::test]
async fn clear_key_resets_has_key() {
    let f = started().await;
    let created = add(&f.engine, EDITABLE, "A", Some(&f.server.base_url()))
        .await
        .unwrap();
    set_key(&f.engine, &created.id, KEY).await.unwrap();
    let request = AiRequest::ClearAiApiKey {
        connection: created.id.clone(),
    };
    let cleared = call_connection(&f.engine, request).await.unwrap();
    assert!(!cleared.has_key);
    assert!(!list(&f.engine).await[0].has_key);
}

/// Un retour à la ligne final est retiré avant le rangement
#[tokio::test]
async fn key_with_trailing_newline_is_trimmed_and_accepted() {
    let f = started().await;
    let created = add(&f.engine, EDITABLE, "A", Some(&f.server.base_url()))
        .await
        .unwrap();
    set_key(&f.engine, &created.id, "  sk-abc\r\n")
        .await
        .unwrap();
    let location = AiSecretRef {
        profile: ProfileId::from_raw("default"),
        connection: created.id.clone(),
    };
    assert_eq!(
        f.secrets.get(&location).unwrap().unwrap().expose(),
        "sk-abc"
    );
}

/// Espace ou caractère de contrôle à l'intérieur, ou plus de 512 octets : refusé, rien n'est rangé
#[tokio::test]
async fn key_with_inner_space_or_control_char_is_refused() {
    let f = started().await;
    let created = add(&f.engine, EDITABLE, "A", Some(&f.server.base_url()))
        .await
        .unwrap();
    let too_long = "k".repeat(513);
    for key in [
        "sk abc",
        "sk\tabc",
        "sk\u{7}abc",
        "sk\nabc",
        too_long.as_str(),
    ] {
        let error = set_key(&f.engine, &created.id, key).await.unwrap_err();
        assert_eq!(error, "invalid_input.ai_key_invalid", "clé {key:?}");
    }
    set_key(&f.engine, &created.id, &"k".repeat(512))
        .await
        .unwrap();
    assert!(list(&f.engine).await[0].has_key);
}

/// Une clé vide, ou faite de blancs, est refusée
#[tokio::test]
async fn empty_key_is_refused() {
    let f = started().await;
    let created = add(&f.engine, EDITABLE, "A", Some(&f.server.base_url()))
        .await
        .unwrap();
    for key in ["", "   ", "\n"] {
        let error = set_key(&f.engine, &created.id, key).await.unwrap_err();
        assert_eq!(error, "invalid_input.ai_key_invalid");
    }
    assert!(!list(&f.engine).await[0].has_key);
}

/// Une session complète sérialisée, erreurs comprises, ne contient jamais la clé
#[tokio::test]
async fn key_not_present_in_any_response_json() {
    let f = started().await;
    let url = f.server.base_url();
    let created = add(&f.engine, EDITABLE, "A", Some(&url)).await.unwrap();
    queue_one_model(&f.server);
    let all = run_session(&f.engine, &created.id, &url, KEY).await;
    assert!(!all.contains(KEY), "la clé fuit : {all}");
}
