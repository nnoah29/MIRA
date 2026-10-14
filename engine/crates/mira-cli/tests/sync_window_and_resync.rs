#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! `mira sync extend-window` et `mira sync resync` en processus contre le faux serveur IMAP.

#[allow(dead_code)]
mod cli_support;

use cli_support::full_bench::{FullBench, USER};

/// Secondes Unix d'il y a `days` jours.
fn days_ago(days: i64) -> i64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    i64::try_from(now).unwrap() - days * 86_400
}

/// Dépose un courrier daté d'il y a `age` jours dans la réception du faux serveur.
fn deliver_aged(bench: &FullBench, subject: &str, age: i64) {
    let raw = format!(
        "From: Bob <bob@example.org>\r\nTo: {USER}\r\nSubject: {subject}\r\nMessage-ID: <{subject}@x>\r\n\r\nCorps\r\n"
    );
    let date = days_ago(age);
    bench
        .store
        .deliver_dated(USER, "INBOX", raw.as_bytes(), &[], date);
}

/// Compte ajouté, première synchro faite : un message récent lu, un message ancien resté sur le serveur.
async fn aged_bench() -> (FullBench, String, String) {
    let bench = FullBench::start().await;
    deliver_aged(&bench, "Recent", 1);
    deliver_aged(&bench, "Ancien", 100);
    let account = bench.add_account().await;
    bench.run_sync_ok(&["sync", "start"]).await;
    let inbox = bench.mailbox_named(&account, "INBOX").await;
    (bench, account, inbox)
}

/// `extend-window` remonte la plage : le message ancien arrive, l'opération est attendue
#[tokio::test]
async fn extend_window_downloads_older_mail() {
    let (bench, _account, inbox) = aged_bench().await;
    assert_eq!(bench.subjects_of(&inbox).await, ["Recent"]);
    let args = [
        "sync",
        "extend-window",
        "--mailbox",
        &inbox,
        "--days",
        "150",
    ];
    let data = bench.run_sync_ok(&args).await;
    assert_eq!(data["data"]["status"]["type"], "finished");
    let mut subjects = bench.subjects_of(&inbox).await;
    subjects.sort();
    assert_eq!(subjects, ["Ancien", "Recent"]);
}

/// `resync` relit la boîte entièrement et ramène le courrier arrivé depuis
#[tokio::test]
async fn resync_rereads_the_whole_mailbox() {
    let (bench, _account, inbox) = aged_bench().await;
    bench.deliver_subject("INBOX", "Nouveau");
    let data = bench
        .run_sync_ok(&["sync", "resync", "--mailbox", &inbox])
        .await;
    assert_eq!(data["data"]["kind"], "resync_mailbox");
    assert_eq!(data["data"]["status"]["type"], "finished");
    let mut subjects = bench.subjects_of(&inbox).await;
    subjects.sort();
    assert_eq!(subjects, ["Nouveau", "Recent"]);
}

/// `start --mailbox` accepte une boîte et la met à jour : le courrier arrivé depuis est lu
#[tokio::test]
async fn start_with_mailbox_scope_brings_that_mailbox_up_to_date() {
    let (bench, _account, inbox) = aged_bench().await;
    bench.deliver_subject("INBOX", "Reçu");
    let args = ["sync", "start", "--mailbox", &inbox];
    let data = bench.run_sync_ok(&args).await;
    assert_eq!(data["data"]["status"]["type"], "finished");
    assert!(
        bench
            .subjects_of(&inbox)
            .await
            .contains(&"Reçu".to_string())
    );
}
