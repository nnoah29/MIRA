#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Enregistrement des pièces jointes : écriture atomique, annulation sans fichier partiel, hors ligne.

mod support;
mod support_api;
mod support_mirror;
mod support_read;

use mira_core::api::{AttachmentTarget, MessageRequest, OperationRequest, Request, Response};
use mira_core::operations::OperationStatus;
use mira_vocabulary::{AttachmentId, CacheState};
use support_api::files::files_under;
use support_read::calls::{run, wait_finished};
use support_read::world::start_world_with_attachment;

/// La pièce jointe est écrite en entier, sans temporaire visible, et son contenu est mis en cache
#[tokio::test]
async fn save_attachment_writes_the_file_atomically() {
    let (world, attachment) = start_world_with_attachment(1_000).await;
    let target = tempfile::tempdir().unwrap();
    let destination = target.path().join("out.bin");
    let operation = world.start_save(&attachment, &destination).await;
    assert_eq!(
        wait_finished(&world.fixture.engine, &operation).await,
        OperationStatus::Finished
    );
    assert_eq!(std::fs::read(&destination).unwrap(), vec![7u8; 1_000]);
    assert_eq!(files_under(target.path()), vec![destination]);
    let stored = world
        .mirror
        .read(|tx| tx.get_attachment(&attachment))
        .unwrap()
        .unwrap();
    assert_eq!(stored.cache, CacheState::Cached);
}

/// Annulée en cours d'écriture, l'opération ne laisse aucun fichier, ni destination ni temporaire
#[tokio::test]
async fn cancelled_save_leaves_no_partial_file() {
    let (world, attachment) = start_world_with_attachment(1_000_000).await;
    let target = tempfile::tempdir().unwrap();
    let destination = target.path().join("big.bin");
    let operation = world.start_save(&attachment, &destination).await;
    tokio::task::yield_now().await;
    assert!(!destination.exists());
    let partial = files_under(world.fixture.home.path());
    assert!(
        partial
            .iter()
            .any(|path| path.to_string_lossy().ends_with(".mira-part"))
    );
    let cancel = Request::Operation(OperationRequest::CancelOperation {
        operation: operation.clone(),
    });
    run(&world.fixture.engine, cancel).await.unwrap();
    assert_eq!(
        wait_finished(&world.fixture.engine, &operation).await,
        OperationStatus::Cancelled
    );
    assert!(files_under(target.path()).is_empty());
    let profile = world.fixture.home.path();
    assert!(
        files_under(profile)
            .iter()
            .all(|path| !path.to_string_lossy().ends_with(".mira-part"))
    );
}

/// Hors ligne, et même après la purge du cache, la pièce jointe se réécrit depuis la source locale
#[tokio::test]
async fn save_works_offline_and_after_cache_purge() {
    let (world, attachment) = start_world_with_attachment(2_000).await;
    world.fixture.imap.shutdown();
    let purge = Request::Maintenance(mira_core::api::MaintenanceRequest::PurgeAttachmentCache);
    assert_eq!(
        run(&world.fixture.engine, purge).await.unwrap(),
        Response::Done
    );
    let target = tempfile::tempdir().unwrap();
    let destination = target.path().join("again.bin");
    let operation = world.start_save(&attachment, &destination).await;
    assert_eq!(
        wait_finished(&world.fixture.engine, &operation).await,
        OperationStatus::Finished
    );
    assert_eq!(std::fs::read(&destination).unwrap().len(), 2_000);
}

/// Une pièce jointe inconnue ou une destination vide sont refusées avant toute opération
#[tokio::test]
async fn save_refuses_unknown_attachment_and_empty_destination() {
    let (world, attachment) = start_world_with_attachment(10).await;
    let save = |attachment: AttachmentId, destination: &str| {
        Request::Message(MessageRequest::SaveAttachment(AttachmentTarget {
            attachment,
            destination: destination.to_string(),
        }))
    };
    let engine = &world.fixture.engine;
    let unknown = run(engine, save(AttachmentId::from_raw("nope"), "/tmp/x"))
        .await
        .unwrap_err();
    assert_eq!(unknown.code, "not_found.attachment");
    let empty = run(engine, save(attachment, " ")).await.unwrap_err();
    assert_eq!(empty.code, "invalid_input.missing_parameter");
}

/// Une destination dans un dossier inexistant fait échouer l'opération sans rien laisser
#[tokio::test]
async fn save_into_missing_directory_fails_cleanly() {
    let (world, attachment) = start_world_with_attachment(10).await;
    let target = tempfile::tempdir().unwrap();
    let destination = target.path().join("absent").join("out.bin");
    let operation = world.start_save(&attachment, &destination).await;
    let status = wait_finished(&world.fixture.engine, &operation).await;
    assert_eq!(
        status,
        OperationStatus::Failed {
            code: "local_storage.io".into()
        }
    );
    assert!(files_under(target.path()).is_empty());
}
