#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Garde-fous de la mémoire : texte vide, trop long, secret ou donnée bancaire ; rien n'est stocké ni embarqué.

mod support_ai_memory;

use std::sync::atomic::Ordering;

use mira_vocabulary::error::codes::invalid_input::{AI_MEMORY_SENSITIVE, AI_MEMORY_TEXT};
use mira_vocabulary::{MemoryKind, MemorySource};
use support_ai_memory::{Setup, setup};

/// Tente de retenir ce texte et rend le code d'erreur, en vérifiant que rien n'a été stocké ni calculé
async fn refused(s: &Setup, text: &str) -> String {
    let error = s
        .store
        .remember(text, MemoryKind::Fact, MemorySource::User)
        .await
        .unwrap_err();
    assert!(
        s.db.read(|tx| tx.list_ai_memories(10, None))
            .unwrap()
            .is_empty()
    );
    assert_eq!(s.embedder.calls.load(Ordering::SeqCst), 0);
    error.code
}

/// Un texte vide ou blanc est refusé
#[tokio::test]
async fn remember_rejects_blank_text() {
    let s = setup("m1");
    assert_eq!(refused(&s, "   \n").await, AI_MEMORY_TEXT.0);
}

/// Un texte de plus de 2 000 caractères est refusé
#[tokio::test]
async fn remember_rejects_text_over_limit() {
    let s = setup("m1");
    assert_eq!(refused(&s, &"é".repeat(2_001)).await, AI_MEMORY_TEXT.0);
}

/// Un texte qui contient une clé secrète est refusé
#[tokio::test]
async fn remember_rejects_secret_like_text() {
    let s = setup("m1");
    let text = "ma clé : sk-ant-api03-AbCdEfGhIjKlMnOpQrStUvWxYz0123456789";
    assert_eq!(refused(&s, text).await, AI_MEMORY_SENSITIVE.0);
}

/// Un numéro d'IBAN est refusé lui aussi
#[tokio::test]
async fn remember_rejects_iban() {
    let s = setup("m1");
    assert_eq!(
        refused(&s, "mon IBAN FR7630006000011234567890189").await,
        AI_MEMORY_SENSITIVE.0
    );
}

/// La limite de 1 000 caractères tient dans le contexte de 512 jetons du modèle local
#[tokio::test]
async fn remember_rejects_text_just_over_limit() {
    let s = setup("m1");
    assert_eq!(refused(&s, &"a".repeat(1_001)).await, AI_MEMORY_TEXT.0);
}
