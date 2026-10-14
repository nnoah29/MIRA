#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Maintenance par la façade : volumes, intégrité, index, exports sans secret (D-29).

mod support;
mod support_api;
mod support_mirror;
mod support_read;

use std::os::unix::fs::PermissionsExt;

use mira_core::api::{MaintenanceRequest, Request, Response, SearchRequest};
use mira_vocabulary::{MessageId, PageRequest};
use support_api::fixture::FOLDERS_PASSWORD;
use support_read::calls::run;
use support_read::seed::add_body;
use support_read::world::start_world;

/// Les volumes du miroir et le contrôle d'intégrité sont rendus
#[tokio::test]
async fn storage_stats_and_integrity_are_reported() {
    let world = start_world().await;
    world.seed_messages(2);
    let engine = &world.fixture.engine;
    let stats = Request::Maintenance(MaintenanceRequest::GetStorageStats);
    let Response::StorageStats(counts) = run(engine, stats).await.unwrap() else {
        panic!("réponse")
    };
    assert_eq!(
        (counts.mailboxes, counts.messages, counts.appearances),
        (1, 2, 2)
    );
    let check = Request::Maintenance(MaintenanceRequest::CheckIntegrity);
    let Response::Integrity(report) = run(engine, check).await.unwrap() else {
        panic!("réponse")
    };
    assert!(report.ok);
}

/// L'index vidé ne trouve plus rien ; sa reconstruction rend la recherche locale
#[tokio::test]
async fn rebuild_search_index_restores_local_search() {
    let world = start_world().await;
    world.seed_messages(1);
    let id = MessageId::from_raw("m1");
    add_body(&world.mirror, &id, "un narval discret", None);
    world.mirror.write(|tx| tx.unindex_message(&id)).unwrap();
    let engine = &world.fixture.engine;
    let search = || {
        Request::Search(SearchRequest::Search {
            query: "narval".into(),
            filters: Default::default(),
            page: PageRequest::default(),
        })
    };
    let Response::Messages(before) = run(engine, search()).await.unwrap() else {
        panic!("réponse")
    };
    assert!(before.items.is_empty());
    let rebuild = Request::Maintenance(MaintenanceRequest::RebuildSearchIndex);
    assert_eq!(run(engine, rebuild).await.unwrap(), Response::Done);
    let Response::Messages(after) = run(engine, search()).await.unwrap() else {
        panic!("réponse")
    };
    assert_eq!(after.items.len(), 1);
}

/// D-29 : l'export du profil contient configuration et intentions, ni secret ni miroir, en 0600
#[tokio::test]
async fn export_profile_has_configuration_but_no_secret_nor_mirror() {
    let world = start_world().await;
    world.seed_messages(2);
    let target = tempfile::tempdir().unwrap();
    let destination = target.path().join("profile.json");
    let request = Request::Maintenance(MaintenanceRequest::ExportProfile {
        destination: destination.to_string_lossy().to_string(),
    });
    let Response::Exported(report) = run(&world.fixture.engine, request).await.unwrap() else {
        panic!("réponse")
    };
    let text = std::fs::read_to_string(&destination).unwrap();
    assert_eq!(report.bytes as usize, text.len());
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(value["format_version"], 1);
    assert_eq!(value["accounts"][0]["id"], world.account.as_str());
    assert_eq!(value["identities"].as_array().unwrap().len(), 1);
    assert!(!text.contains(FOLDERS_PASSWORD) && !text.contains("Sujet m1"));
    assert_eq!(
        std::fs::metadata(&destination)
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    assert_eq!(std::fs::read_dir(target.path()).unwrap().count(), 1);
}

/// Le diagnostic donne versions, volumes et état des comptes, sans adresse ni secret
#[tokio::test]
async fn export_diagnostic_has_no_secret_and_no_address() {
    let world = start_world().await;
    let target = tempfile::tempdir().unwrap();
    let destination = target.path().join("diagnostic.json");
    let request = Request::Maintenance(MaintenanceRequest::ExportDiagnostic {
        destination: destination.to_string_lossy().to_string(),
    });
    run(&world.fixture.engine, request).await.unwrap();
    let text = std::fs::read_to_string(&destination).unwrap();
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(value["integrity_ok"], true);
    assert_eq!(value["contract_version"]["major"], 1);
    assert_eq!(value["accounts"].as_array().unwrap().len(), 1);
    assert!(!text.contains(FOLDERS_PASSWORD) && !text.contains(world.user()));
}

/// Un export vers un dossier inexistant échoue sans créer de fichier
#[tokio::test]
async fn export_to_missing_directory_fails_without_file() {
    let world = start_world().await;
    let target = tempfile::tempdir().unwrap();
    let destination = target.path().join("absent").join("profile.json");
    let request = Request::Maintenance(MaintenanceRequest::ExportProfile {
        destination: destination.to_string_lossy().to_string(),
    });
    let error = run(&world.fixture.engine, request).await.unwrap_err();
    assert_eq!(error.code, "local_storage.io");
    assert!(!target.path().join("absent").exists());
}
