#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Téléchargement d'un fichier épinglé : installation, reprise, redirection, somme et taille vérifiées.

use mira_core::ai::local::download_file;
use mira_core::ports::FixedResources;

mod local_support;
use local_support::download::{content, dest_in, job, no_cancel, sha_hex};
use local_support::file_host::FileHost;

/// Un téléchargement complet installe le fichier sous son nom final et laisse un `.part` absent
#[tokio::test]
async fn download_succeeds_and_installs_the_file() {
    let bytes = content(50_000);
    let host = FileHost::start(bytes.clone()).await;
    let dir = tempfile::tempdir().unwrap();
    let dest = dest_in(&dir);
    let sha = sha_hex(&bytes);
    let res = FixedResources::new(Some(8000), Some(u64::MAX));
    let url = host.url();
    download_file(
        &job(&url, &bytes, &sha, &dest, &res),
        &|_, _| {},
        &no_cancel(),
    )
    .await
    .unwrap();
    assert_eq!(std::fs::read(&dest).unwrap(), bytes);
    assert!(!dest.with_extension("bin.part").exists());
}

/// Après une coupure, un second appel reprend par `Range` et finit avec la bonne somme
#[tokio::test]
async fn download_resumes_after_a_cut_and_ends_with_the_right_hash() {
    let bytes = content(60_000);
    let host = FileHost::start(bytes.clone()).await;
    let dir = tempfile::tempdir().unwrap();
    let dest = dest_in(&dir);
    let sha = sha_hex(&bytes);
    let res = FixedResources::new(Some(8000), Some(u64::MAX));
    let url = host.url();
    host.cut_next_response_after(20_000);
    let first = download_file(
        &job(&url, &bytes, &sha, &dest, &res),
        &|_, _| {},
        &no_cancel(),
    )
    .await;
    assert!(first.is_err());
    assert!(!dest.exists());
    download_file(
        &job(&url, &bytes, &sha, &dest, &res),
        &|_, _| {},
        &no_cancel(),
    )
    .await
    .unwrap();
    assert_eq!(std::fs::read(&dest).unwrap(), bytes);
    assert_eq!(host.last_blob_range().as_deref(), Some("bytes=20000-"));
}

/// La redirection est suivie et aucune requête ne porte d'en-tête `Authorization`
#[tokio::test]
async fn download_follows_redirect_without_authorization_header() {
    let bytes = content(10_000);
    let host = FileHost::start(bytes.clone()).await;
    let dir = tempfile::tempdir().unwrap();
    let dest = dest_in(&dir);
    let res = FixedResources::new(Some(8000), Some(u64::MAX));
    let url = host.url();
    download_file(
        &job(&url, &bytes, &sha_hex(&bytes), &dest, &res),
        &|_, _| {},
        &no_cancel(),
    )
    .await
    .unwrap();
    let seen = host.requests();
    assert!(seen.iter().any(|r| r.path == "/file") && seen.iter().any(|r| r.path == "/blob"));
    assert!(seen.iter().all(|r| !r.has_authorization));
}

/// Une somme différente supprime le fichier et lève le code de somme invalide
#[tokio::test]
async fn checksum_mismatch_deletes_the_file_and_errors() {
    let bytes = content(10_000);
    let host = FileHost::start(bytes.clone()).await;
    let dir = tempfile::tempdir().unwrap();
    let dest = dest_in(&dir);
    let wrong = sha_hex(b"autre chose");
    let res = FixedResources::new(Some(8000), Some(u64::MAX));
    let url = host.url();
    let error = download_file(
        &job(&url, &bytes, &wrong, &dest, &res),
        &|_, _| {},
        &no_cancel(),
    )
    .await
    .unwrap_err();
    assert_eq!(error.code, "server.ai_local_checksum_mismatch");
    assert!(!dest.exists() && !dest.with_extension("bin.part").exists());
}

/// Un fichier plus long que la taille épinglée est refusé comme une somme invalide
#[tokio::test]
async fn size_mismatch_is_refused() {
    let bytes = content(10_000);
    let host = FileHost::start(bytes.clone()).await;
    let dir = tempfile::tempdir().unwrap();
    let dest = dest_in(&dir);
    let res = FixedResources::new(Some(8000), Some(u64::MAX));
    let url = host.url();
    let short = &bytes[..9_000];
    let error = download_file(
        &job(&url, short, &sha_hex(&bytes), &dest, &res),
        &|_, _| {},
        &no_cancel(),
    )
    .await
    .unwrap_err();
    assert_eq!(error.code, "server.ai_local_checksum_mismatch");
    assert!(!dest.exists());
}

/// Un contenu changé en amont pendant la reprise est détecté par la somme finale
#[tokio::test]
async fn changed_content_on_resume_is_caught_by_the_hash() {
    let bytes = content(30_000);
    let host = FileHost::start(bytes.clone()).await;
    let dir = tempfile::tempdir().unwrap();
    let dest = dest_in(&dir);
    let sha = sha_hex(&bytes);
    let res = FixedResources::new(Some(8000), Some(u64::MAX));
    let url = host.url();
    host.cut_next_response_after(10_000);
    let _ = download_file(
        &job(&url, &bytes, &sha, &dest, &res),
        &|_, _| {},
        &no_cancel(),
    )
    .await;
    let mut other = bytes.clone();
    other.iter_mut().skip(10_000).for_each(|b| *b ^= 0xff);
    host.serve_other_content_on_range(other);
    let error = download_file(
        &job(&url, &bytes, &sha, &dest, &res),
        &|_, _| {},
        &no_cancel(),
    )
    .await
    .unwrap_err();
    assert_eq!(error.code, "server.ai_local_checksum_mismatch");
    assert!(!dest.exists());
}
