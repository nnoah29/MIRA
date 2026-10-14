#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Suppression d'un compte (S14) : les fichiers de cache des pièces jointes partent avec lui.

mod support;
mod support_api;
mod support_mirror;
mod support_read;

use std::path::Path;

use mira_core::api::{AccountRequest, Request, Response};
use mira_core::operations::OperationStatus;
use support_read::calls::{run, wait_finished};
use support_read::world::start_world_with_attachment;

/// Supprimer le compte efface le fichier de cache de la pièce jointe, pas seulement sa ligne en base
#[tokio::test]
async fn regression_deleting_an_account_removes_its_attachment_cache_files() {
    let (world, attachment) = start_world_with_attachment(2_000).await;
    let target = tempfile::tempdir().unwrap();
    let destination = target.path().join("out.bin");
    let operation = world.start_save(&attachment, &destination).await;
    let engine = &world.fixture.engine;
    assert_eq!(
        wait_finished(engine, &operation).await,
        OperationStatus::Finished
    );
    let cached = world
        .mirror
        .read(|tx| tx.attachment_cache_path(&attachment))
        .unwrap()
        .unwrap();
    assert!(Path::new(&cached).exists());
    let delete = Request::Account(AccountRequest::DeleteAccount {
        account: world.account.clone(),
    });
    assert_eq!(run(engine, delete).await.unwrap(), Response::Done);
    assert!(!Path::new(&cached).exists(), "le cache a survécu au compte");
    assert!(
        destination.exists(),
        "le fichier de l'utilisateur est intact"
    );
}
