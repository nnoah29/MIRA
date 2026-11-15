#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Installation du modèle d'embeddings : téléchargement vérifié, refus d'une somme fausse, état installé.

mod local_support;

use local_support::download::{content, no_cancel, sha_hex};
use local_support::file_host::FileHost;
use local_support::test_model;
use mira_core::ai::local::LocalPaths;
use mira_core::ai::memory::{install_embedding_model, is_embedding_installed};
use mira_core::ports::FixedResources;
use mira_vocabulary::{AiLocalModel, LocalTier};

/// Fiche de modèle d'embeddings dont le contenu, l'adresse et la somme sont ceux du faux serveur
fn model_for(bytes: &[u8], url: &str, sha: &str) -> AiLocalModel {
    let mut model = test_model("embed-test", LocalTier::Mini, false);
    model.url = url.to_string();
    model.sha256 = sha.to_string();
    model.size = bytes.len() as u64;
    model
}

/// Le téléchargement depuis le faux serveur installe le fichier sous `embeddings/` avec le bon contenu
#[tokio::test]
async fn install_downloads_from_fake_server_and_verifies() {
    let bytes = content(40_000);
    let host = FileHost::start(bytes.clone()).await;
    let dir = tempfile::tempdir().unwrap();
    let paths = LocalPaths::new(dir.path());
    let model = model_for(&bytes, &host.url(), &sha_hex(&bytes));
    let res = FixedResources::new(Some(8000), Some(u64::MAX));
    let file = install_embedding_model(&paths, &model, &res, &|_, _| {}, &no_cancel())
        .await
        .unwrap();
    assert_eq!(file, paths.embedding_file(&model));
    assert!(file.starts_with(paths.root().join("embeddings")));
    assert_eq!(std::fs::read(&file).unwrap(), bytes);
}

/// Une somme SHA-256 fausse est refusée et aucun fichier final n'existe
#[tokio::test]
async fn install_rejects_wrong_sha256() {
    let bytes = content(20_000);
    let host = FileHost::start(bytes.clone()).await;
    let dir = tempfile::tempdir().unwrap();
    let paths = LocalPaths::new(dir.path());
    let model = model_for(&bytes, &host.url(), &"a".repeat(64));
    let res = FixedResources::new(Some(8000), Some(u64::MAX));
    let result = install_embedding_model(&paths, &model, &res, &|_, _| {}, &no_cancel()).await;
    assert!(result.is_err());
    assert!(!paths.embedding_file(&model).exists());
    assert!(!is_embedding_installed(&paths, &model));
}

/// Le modèle n'est installé qu'après un téléchargement réussi
#[tokio::test]
async fn is_installed_false_before_and_true_after() {
    let bytes = content(10_000);
    let host = FileHost::start(bytes.clone()).await;
    let dir = tempfile::tempdir().unwrap();
    let paths = LocalPaths::new(dir.path());
    let model = model_for(&bytes, &host.url(), &sha_hex(&bytes));
    assert!(!is_embedding_installed(&paths, &model));
    let res = FixedResources::new(Some(8000), Some(u64::MAX));
    install_embedding_model(&paths, &model, &res, &|_, _| {}, &no_cancel())
        .await
        .unwrap();
    assert!(is_embedding_installed(&paths, &model));
}
