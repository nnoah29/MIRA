#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! La liste des modèles d'une connexion locale est celle des fichiers installés, sans lancer le serveur.

use mira_core::api::{AiRequest, AiResponse, Request, Response};
use mira_vocabulary::AiConnectionId;

mod local_chat_support;
mod local_support;
mod support_api;
use local_chat_support::fixture::LocalFixture;

/// Demande la liste des modèles de `local-1`, en forçant le rafraîchissement
async fn list(fixture: &LocalFixture) -> Vec<String> {
    let request = Request::Ai(AiRequest::ListAiModels {
        connection: AiConnectionId::from_raw("local-1"),
        refresh: true,
    });
    match fixture.engine.execute(request).await.unwrap() {
        Response::Ai(AiResponse::Models(list)) => list.models.into_iter().map(|m| m.id).collect(),
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Les deux fichiers posés donnent deux modèles, et aucun serveur n'est lancé
#[tokio::test]
async fn models_list_is_the_installed_models_and_does_not_launch_the_server() {
    let fixture = LocalFixture::start(300);
    fixture.add_connection(None);
    let mut ids = list(&fixture).await;
    ids.sort();
    assert_eq!(ids, ["big", "mini"]);
    assert_eq!(fixture.starts(), 0);
}

/// Un modèle supprimé du disque disparaît de la liste, même rafraîchie ou non
#[tokio::test]
async fn a_removed_model_file_leaves_the_list() {
    let fixture = LocalFixture::start(300);
    fixture.add_connection(None);
    std::fs::remove_file(fixture.root.join("models").join("big.bin")).unwrap();
    assert_eq!(list(&fixture).await, ["mini"]);
}

/// Un trousseau vide ne gêne pas : une connexion locale n'a ni clé ni consentement
#[tokio::test]
async fn local_connection_needs_neither_key_nor_consent() {
    let fixture = LocalFixture::start(300);
    let connection = fixture.add_connection(Some("mini"));
    assert!(!connection.has_key && connection.consent_at.is_none());
    assert_eq!(list(&fixture).await.len(), 2);
}
