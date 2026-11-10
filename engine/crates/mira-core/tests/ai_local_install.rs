#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Installation du serveur : téléchargement vérifié puis extraction, plateforme choisie, rien de refait.

use std::os::unix::fs::PermissionsExt;

use mira_core::ai::local::{ServerInstall, install_server, select_build};
use mira_core::ports::FixedResources;
use mira_vocabulary::AiLocalServerBuild;

mod local_support;
use local_support::archives::{Entry, tar_gz};
use local_support::download::{no_cancel, sha_hex};
use local_support::file_host::FileHost;

/// Fiche d'archive de test pour la plateforme demandée
fn build_for(platform: &str, url: &str, archive: &[u8]) -> AiLocalServerBuild {
    AiLocalServerBuild {
        platform: platform.into(),
        url: url.into(),
        sha256: sha_hex(archive),
        size: archive.len() as u64,
        executable: "model-server".into(),
    }
}

/// Archive d'un serveur et de sa bibliothèque
fn server_archive() -> Vec<u8> {
    tar_gz(&[
        Entry::file("b/bin/model-server", b"EXE"),
        Entry::file("b/bin/lib.so", b"L"),
    ])
}

/// Télécharge, vérifie, extrait, rend l'exécutable (0755) et supprime l'archive
#[tokio::test]
async fn install_server_downloads_verifies_extracts_and_returns_the_executable() {
    let archive = server_archive();
    let host = FileHost::start(archive.clone()).await;
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("server").join("b12224");
    let build = build_for("linux-x86_64", &host.url(), &archive);
    let res = FixedResources::new(Some(8000), Some(u64::MAX));
    let job = ServerInstall {
        build: &build,
        root: &root,
        resources: &res,
    };
    let exe = install_server(&job, &|_, _| {}, &no_cancel())
        .await
        .unwrap();
    assert_eq!(exe, root.join("b/bin/model-server"));
    assert_eq!(
        std::fs::metadata(&exe).unwrap().permissions().mode() & 0o777,
        0o755
    );
    assert!(!dir.path().join("server").join("b12224.tar.gz").exists());
}

/// Une somme d'archive fausse est refusée avant toute extraction
#[tokio::test]
async fn wrong_archive_hash_is_refused_before_extraction() {
    let archive = server_archive();
    let host = FileHost::start(archive.clone()).await;
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("b12224");
    let mut build = build_for("linux-x86_64", &host.url(), &archive);
    build.sha256 = sha_hex(b"autre");
    let res = FixedResources::new(Some(8000), Some(u64::MAX));
    let job = ServerInstall {
        build: &build,
        root: &root,
        resources: &res,
    };
    let error = install_server(&job, &|_, _| {}, &no_cancel())
        .await
        .unwrap_err();
    assert_eq!(error.code, "server.ai_local_checksum_mismatch");
    assert!(!root.exists());
}

/// Un serveur déjà installé n'est pas retéléchargé
#[tokio::test]
async fn an_installed_server_is_not_downloaded_again() {
    let archive = server_archive();
    let host = FileHost::start(archive.clone()).await;
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("b12224");
    let build = build_for("linux-x86_64", &host.url(), &archive);
    let res = FixedResources::new(Some(8000), Some(u64::MAX));
    let job = ServerInstall {
        build: &build,
        root: &root,
        resources: &res,
    };
    install_server(&job, &|_, _| {}, &no_cancel())
        .await
        .unwrap();
    let before = host.requests().len();
    install_server(&job, &|_, _| {}, &no_cancel())
        .await
        .unwrap();
    assert_eq!(host.requests().len(), before);
}

/// Une plateforme absente du descripteur est une erreur explicite
#[test]
fn unsupported_platform_is_an_explicit_error() {
    let builds = [build_for("linux-x86_64", "https://x.test/a", b"a")];
    assert!(select_build(&builds, "linux-x86_64").is_ok());
    let error = select_build(&builds, "plan9-mips").unwrap_err();
    assert_eq!(
        error.code,
        "incompatible_state.ai_local_unsupported_platform"
    );
}
