#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Contrat 1.4 : renommer, supprimer et effacer les conversations de l'assistant IA.

mod support_ai_conversations;
mod support_api;

use mira_vocabulary::Timestamp;
use support_ai_conversations::changes::{clear, delete, rename};
use support_ai_conversations::{get, list, save, start};

/// Renommer change le titre et le contenu reste, sans bouger l'ordre ni la date de mise à jour
#[tokio::test]
async fn rename_changes_title_but_not_order() {
    let f = start();
    save(&f.engine, "a", "A", "{\"k\":1}").await.unwrap();
    f.clock.advance_millis(10);
    save(&f.engine, "b", "B", "{}").await.unwrap();
    f.clock.advance_millis(10);
    let meta = rename(&f.engine, "a", "  Nouveau titre  ").await.unwrap();
    assert_eq!(meta.title, "Nouveau titre");
    assert_eq!(meta.updated_at, Timestamp::from_millis(1_000));
    let ids: Vec<String> = list(&f.engine, 50, None)
        .await
        .unwrap()
        .into_iter()
        .map(|m| m.id)
        .collect();
    assert_eq!(ids, ["b", "a"]);
    assert_eq!(get(&f.engine, "a").await.unwrap().payload, "{\"k\":1}");
}

/// Un titre invalide ou une conversation inconnue sont refusés
#[tokio::test]
async fn rename_refuses_invalid_title_and_unknown_id() {
    let f = start();
    save(&f.engine, "a", "A", "{}").await.unwrap();
    for title in ["", "   ", &"x".repeat(81)] {
        let refused = rename(&f.engine, "a", title).await;
        assert_eq!(refused.unwrap_err(), "invalid_input.ai_conversation_title");
    }
    let unknown = rename(&f.engine, "absente", "T").await;
    assert_eq!(unknown.unwrap_err(), "not_found.ai_conversation");
    assert_eq!(get(&f.engine, "a").await.unwrap().meta.title, "A");
}

/// Supprimer retire seulement cette conversation ; une inconnue donne `not_found`
#[tokio::test]
async fn delete_removes_only_that_one_and_refuses_unknown() {
    let f = start();
    save(&f.engine, "a", "A", "{}").await.unwrap();
    save(&f.engine, "b", "B", "{}").await.unwrap();
    delete(&f.engine, "a").await.unwrap();
    let left = list(&f.engine, 50, None).await.unwrap();
    assert_eq!(left.len(), 1);
    assert_eq!(left[0].id, "b");
    let again = delete(&f.engine, "a").await;
    assert_eq!(again.unwrap_err(), "not_found.ai_conversation");
}

/// Tout effacer répond `Done`, vide la liste, et n'échoue pas quand il n'y a rien
#[tokio::test]
async fn clear_removes_everything_and_answers_done() {
    let f = start();
    save(&f.engine, "a", "A", "{}").await.unwrap();
    save(&f.engine, "b", "B", "{}").await.unwrap();
    clear(&f.engine).await.unwrap();
    assert!(list(&f.engine, 50, None).await.unwrap().is_empty());
    clear(&f.engine).await.unwrap();
}
