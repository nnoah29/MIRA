#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Contrat 1.4 : interrupteur « Conserver l'historique » des conversations de l'assistant IA.

mod support_ai_conversations;
mod support_api;

use support_ai_conversations::changes::delete;
use support_ai_conversations::history::{enabled, set_enabled};
use support_ai_conversations::{get, list, save, start};

/// L'historique est activé par défaut
#[tokio::test]
async fn enabled_by_default() {
    let f = start();
    assert!(enabled(&f.engine).await);
}

/// Coupé, l'enregistrement est refusé et rien n'est écrit
#[tokio::test]
async fn disabling_refuses_save() {
    let f = start();
    assert!(!set_enabled(&f.engine, false).await.unwrap());
    let refused = save(&f.engine, "a", "A", "{}").await;
    assert_eq!(
        refused.unwrap_err(),
        "incompatible_state.ai_history_disabled"
    );
    assert!(list(&f.engine, 50, None).await.unwrap().is_empty());
}

/// Couper l'historique garde les conversations existantes lisibles et supprimables
#[tokio::test]
async fn disabling_keeps_existing_conversations_readable_and_deletable() {
    let f = start();
    save(&f.engine, "a", "A", "{\"k\":1}").await.unwrap();
    set_enabled(&f.engine, false).await.unwrap();
    assert_eq!(list(&f.engine, 50, None).await.unwrap().len(), 1);
    assert_eq!(get(&f.engine, "a").await.unwrap().payload, "{\"k\":1}");
    delete(&f.engine, "a").await.unwrap();
    assert!(list(&f.engine, 50, None).await.unwrap().is_empty());
}

/// Réactivé, l'enregistrement repart
#[tokio::test]
async fn re_enabling_allows_save_again() {
    let f = start();
    set_enabled(&f.engine, false).await.unwrap();
    assert!(set_enabled(&f.engine, true).await.unwrap());
    save(&f.engine, "a", "A", "{}").await.unwrap();
    assert_eq!(list(&f.engine, 50, None).await.unwrap().len(), 1);
}

/// Le réglage survit à la fermeture et à la réouverture du profil
#[tokio::test]
async fn setting_survives_reopening_the_profile() {
    let home = tempfile::tempdir().unwrap();
    let first = support_api::open(home.path());
    set_enabled(&first, false).await.unwrap();
    first.shutdown().unwrap();
    let second = support_api::open(home.path());
    assert!(!enabled(&second).await);
    second.shutdown().unwrap();
}
