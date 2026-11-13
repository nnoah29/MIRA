#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Installation du serveur depuis une archive `.zip` : l'extension de l'adresse choisit l'extraction.

use mira_core::ai::local::{ServerInstall, install_server};
use mira_core::ports::FixedResources;
use mira_vocabulary::AiLocalServerBuild;

mod local_support;
use local_support::download::{no_cancel, sha_hex};
use local_support::file_host::FileHost;
use local_support::zip_archives::{ZipEntry, zip_bytes};

/// Fiche d'une archive `.zip` servie par l'hôte de test
fn zip_build(url: &str, archive: &[u8]) -> AiLocalServerBuild {
    AiLocalServerBuild {
        platform: "windows-x86_64".into(),
        url: format!("{url}/server.zip"),
        sha256: sha_hex(archive),
        size: archive.len() as u64,
        executable: "model-server.exe".into(),
    }
}

/// Télécharge, vérifie, extrait le zip, trouve l'exécutable et supprime l'archive
#[tokio::test]
async fn install_server_handles_a_zip_archive() {
    let archive = zip_bytes(&[
        ZipEntry::file("bin/model-server.exe", b"EXE"),
        ZipEntry::file("bin/lib.dll", b"L"),
    ]);
    let host = FileHost::start(archive.clone()).await;
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("server").join("b12224");
    let build = zip_build(&host.url(), &archive);
    let res = FixedResources::new(Some(8000), Some(u64::MAX));
    let job = ServerInstall {
        build: &build,
        root: &root,
        resources: &res,
    };
    let exe = install_server(&job, &|_, _| {}, &no_cancel())
        .await
        .unwrap();
    assert_eq!(exe, root.join("bin/model-server.exe"));
    assert_eq!(std::fs::read(&exe).unwrap(), b"EXE");
    assert!(!dir.path().join("server").join("b12224.zip").exists());
}
