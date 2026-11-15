#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Le modèle d'embeddings de la mémoire est épinglé par adresse sécurisée, taille et somme SHA-256.

use mira_ai_local::{embedding_model, embedding_server_args};

/// La fiche porte une somme de 64 hexadécimaux, une taille plafonnée et une adresse https
#[test]
fn embedding_model_is_pinned() {
    let model = embedding_model();
    assert!(!model.id.is_empty());
    assert_eq!(model.sha256.len(), 64);
    assert!(
        model
            .sha256
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    );
    assert!(model.size > 0 && model.size <= 300 * 1024 * 1024);
    assert!(model.url.starts_with("https://"));
    assert!(model.url.ends_with(&model.file_name));
}

/// Le modèle d'embeddings ne se confond avec aucun palier de discussion
#[test]
fn embedding_model_is_not_a_chat_model() {
    let embedding = embedding_model();
    let chat = mira_ai_local::LocalAiProvider;
    let descriptor = mira_vocabulary::AiProvider::descriptor(&chat);
    let runtime = descriptor.local_runtime.unwrap();
    assert!(runtime.models.iter().all(|m| m.id != embedding.id));
}

/// Le serveur d'embeddings est lancé en mode embeddings, pooling CLS explicite et contexte de 512 jetons
#[test]
fn embedding_server_args_force_cls_pooling() {
    let args = embedding_server_args();
    let at = |flag: &str| args.iter().position(|a| a == flag).unwrap();
    assert!(args.contains(&"--embedding".to_string()));
    assert_eq!(args[at("--pooling") + 1], "cls");
    assert_eq!(args[at("-c") + 1], "512");
    assert_eq!(args[at("-ub") + 1], "512");
}

/// La fiche du runtime local porte le modèle d'embeddings et ses arguments pour le cœur
#[test]
fn local_runtime_carries_the_embedding_model_and_args() {
    let descriptor = mira_vocabulary::AiProvider::descriptor(&mira_ai_local::LocalAiProvider);
    let embedding = descriptor.local_runtime.unwrap().embedding.unwrap();
    assert_eq!(embedding.model, embedding_model());
    assert_eq!(embedding.server_args, embedding_server_args());
}
