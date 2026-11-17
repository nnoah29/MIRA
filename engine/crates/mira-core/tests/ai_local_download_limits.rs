#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Téléchargement d'un fichier épinglé : espace disque, annulation, progression, clair refusé.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use mira_core::ai::local::{DownloadJob, download_file};
use mira_core::ports::FixedResources;

mod local_support;
use local_support::download::{content, dest_in, no_cancel, sha_hex};
use local_support::file_host::FileHost;

/// Trop peu d'espace libre : refus avant toute requête réseau
#[tokio::test]
async fn disk_space_too_low_refuses_before_any_request() {
    let bytes = content(10_000);
    let host = FileHost::start(bytes.clone()).await;
    let dir = tempfile::tempdir().unwrap();
    let dest = dest_in(&dir);
    let res = FixedResources::new(Some(8000), Some(5_000));
    let url = host.url();
    let sha = sha_hex(&bytes);
    let job = DownloadJob {
        url: &url,
        sha256: &sha,
        size: 10_000,
        dest: &dest,
        resources: &res,
    };
    let error = download_file(&job, &|_, _| {}, &no_cancel())
        .await
        .unwrap_err();
    assert_eq!(error.code, "incompatible_state.ai_local_disk_space");
    assert!(host.requests().is_empty());
}

/// L'annulation arrête le transfert, garde le `.part` pour une reprise et rend `cancelled.by_user`
#[tokio::test]
async fn cancel_stops_and_keeps_the_part_file() {
    let bytes = content(2_000_000);
    let host = FileHost::start(bytes.clone()).await;
    let dir = tempfile::tempdir().unwrap();
    let dest = dest_in(&dir);
    let res = FixedResources::new(Some(8000), Some(u64::MAX));
    let (url, sha) = (host.url(), sha_hex(&bytes));
    let flag = Arc::new(AtomicBool::new(false));
    let raise = flag.clone();
    let job = DownloadJob {
        url: &url,
        sha256: &sha,
        size: 2_000_000,
        dest: &dest,
        resources: &res,
    };
    let on_progress = move |done: u64, _total: u64| {
        if done > 100_000 {
            raise.store(true, Ordering::SeqCst);
        }
    };
    let error = download_file(&job, &on_progress, &flag).await.unwrap_err();
    assert_eq!(error.code, "cancelled.by_user");
    assert!(!dest.exists() && dest.with_extension("bin.part").exists());
}

/// La progression ne recule jamais et finit à `total`
#[tokio::test]
async fn progress_reports_increasing_done_up_to_total() {
    let bytes = content(300_000);
    let host = FileHost::start(bytes.clone()).await;
    let dir = tempfile::tempdir().unwrap();
    let dest = dest_in(&dir);
    let res = FixedResources::new(Some(8000), Some(u64::MAX));
    let (url, sha) = (host.url(), sha_hex(&bytes));
    let seen = Arc::new(Mutex::new(Vec::new()));
    let sink = seen.clone();
    let job = DownloadJob {
        url: &url,
        sha256: &sha,
        size: 300_000,
        dest: &dest,
        resources: &res,
    };
    let on_progress = move |done: u64, total: u64| sink.lock().unwrap().push((done, total));
    download_file(&job, &on_progress, &no_cancel())
        .await
        .unwrap();
    let seen = seen.lock().unwrap();
    assert!(seen.windows(2).all(|w| w[0].0 <= w[1].0));
    assert_eq!(*seen.last().unwrap(), (300_000, 300_000));
}

/// Du HTTP en clair vers un hôte distant est refusé, sans requête
#[tokio::test]
async fn plain_http_is_refused_except_loopback() {
    let dir = tempfile::tempdir().unwrap();
    let dest = dest_in(&dir);
    let res = FixedResources::new(Some(8000), Some(u64::MAX));
    let sha = sha_hex(b"x");
    let url = "http://models.example.org/m.bin";
    let job = DownloadJob {
        url,
        sha256: &sha,
        size: 1,
        dest: &dest,
        resources: &res,
    };
    let error = download_file(&job, &|_, _| {}, &no_cancel())
        .await
        .unwrap_err();
    assert_eq!(error.code, "tls.plaintext_refused");
}
