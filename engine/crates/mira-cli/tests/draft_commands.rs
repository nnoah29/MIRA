#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Commandes `mira draft …` : rédaction hors ligne, pièces jointes, réponse, envoi contre le faux SMTP.

#[allow(dead_code)]
mod cli_support;

use cli_support::full_bench::FullBench;

/// Cycle de rédaction : créer, modifier, lire, lister, supprimer, tout hors ligne
#[tokio::test]
async fn draft_create_update_get_list_delete_cycle() {
    let (bench, account) = FullBench::start_seeded().await;
    bench.imap.shutdown();
    let created = bench.run_ok(&["draft", "create", &account]).await;
    let id = created["data"]["data"]["id"].as_str().unwrap().to_string();
    let update = [
        "draft",
        "update",
        &id,
        "--to",
        "Bob <bob@example.org>",
        "--cc",
        "carla@example.org",
        "--subject",
        "Bonjour Bob",
        "--body",
        "Le texte",
    ];
    bench.run_ok(&update).await;
    let got = bench.run_ok(&["draft", "get", &id]).await;
    assert_eq!(got["data"]["data"]["subject"], "Bonjour Bob");
    assert_eq!(got["data"]["data"]["to"][0]["name"], "Bob");
    assert_eq!(got["data"]["data"]["cc"][0]["address"], "carla@example.org");
    let human = bench.run(&["draft", "get", &id]).await;
    assert!(human.stdout.contains("Objet   : Bonjour Bob") && human.stdout.ends_with("Le texte\n"));
    let list = bench.run_ok(&["draft", "list", &account]).await;
    assert_eq!(list["data"]["data"].as_array().unwrap().len(), 1);
    bench.run_ok(&["draft", "delete", &id]).await;
    assert_eq!(bench.run(&["draft", "get", &id]).await.code, 11);
}

/// Pièces jointes : `attach` joint un fichier, `detach` le retire ; position inconnue refusée
#[tokio::test]
async fn draft_attach_and_detach_a_file() {
    let (bench, account) = FullBench::start_seeded().await;
    let created = bench.run_ok(&["draft", "create", &account]).await;
    let id = created["data"]["data"]["id"].as_str().unwrap().to_string();
    let folder = tempfile::tempdir().unwrap();
    let file = folder.path().join("notes.txt");
    std::fs::write(&file, b"contenu").unwrap();
    let path = file.to_str().unwrap();
    let attached = bench.run_ok(&["draft", "attach", &id, path]).await;
    assert_eq!(
        attached["data"]["data"]["attachments"][0]["filename"],
        "notes.txt"
    );
    let detached = bench.run_ok(&["draft", "detach", &id, "0"]).await;
    assert!(
        detached["data"]["data"]["attachments"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(bench.run(&["draft", "detach", &id, "3"]).await.code, 10);
}

/// `draft create --reply` : brouillon de réponse relié au message et adressé à son expéditeur
#[tokio::test]
async fn draft_reply_is_addressed_to_the_sender() {
    let (bench, account) = FullBench::start_seeded().await;
    let created = bench
        .run_ok(&["draft", "create", &account, "--reply", "m1"])
        .await;
    let draft = &created["data"]["data"];
    assert_eq!(draft["in_reply_to"], "m1");
    assert_eq!(draft["to"][0]["address"], "bob@example.org");
    assert_eq!(draft["subject"], "Re: Sujet m1");
    let conflict = bench
        .run(&[
            "draft",
            "create",
            &account,
            "--reply",
            "m1",
            "--forward",
            "m2",
        ])
        .await;
    assert_eq!(conflict.code, 10);
}

/// `draft send --wait` : le message part par le faux SMTP, la boîte d'envoi se vide
#[tokio::test]
async fn draft_send_wait_delivers_through_smtp() {
    let (bench, account) = FullBench::start_seeded().await;
    let created = bench.run_ok(&["draft", "create", &account]).await;
    let id = created["data"]["data"]["id"].as_str().unwrap().to_string();
    let update = [
        "draft",
        "update",
        &id,
        "--to",
        "bob@example.org",
        "--subject",
        "Salut",
        "--body",
        "Texte",
    ];
    bench.run_ok(&update).await;
    let pump = bench.pump_clock();
    let sent = bench.run_ok(&["draft", "send", &id, "--wait"]).await;
    pump.abort();
    assert_eq!(sent["data"]["type"], "action");
    let received = bench.smtp.received();
    assert_eq!(received.len(), 1);
    assert_eq!(received[0].recipients, ["bob@example.org"]);
    let outbox = bench.run_ok(&["draft", "outbox", &account]).await;
    assert!(outbox["data"]["data"].as_array().unwrap().is_empty());
    assert_eq!(bench.store.messages("alice@folders.test", "Sent").len(), 1);
}

/// `draft send` puis `cancel-send` pendant le délai : rien ne part, le brouillon redevient éditable
#[tokio::test]
async fn draft_send_then_cancel_send_during_the_delay() {
    let (bench, account) = FullBench::start_seeded().await;
    let created = bench.run_ok(&["draft", "create", &account]).await;
    let id = created["data"]["data"]["id"].as_str().unwrap().to_string();
    bench
        .run_ok(&["draft", "update", &id, "--to", "bob@example.org"])
        .await;
    let sent = bench.run_ok(&["draft", "send", &id]).await;
    let action = sent["data"]["data"].as_str().unwrap().to_string();
    let outbox = bench.run_ok(&["draft", "outbox", &account]).await;
    assert_eq!(outbox["data"]["data"][0]["state"], "pending");
    assert_eq!(
        bench
            .run(&["draft", "outbox", &account])
            .await
            .stdout
            .lines()
            .count(),
        2
    );
    bench.run_ok(&["draft", "cancel-send", &action]).await;
    let after = bench.run_ok(&["draft", "outbox", &account]).await;
    assert!(after["data"]["data"].as_array().unwrap().is_empty());
    let draft = bench.run_ok(&["draft", "get", &id]).await;
    assert_eq!(draft["data"]["data"]["state"]["type"], "editing");
    assert!(bench.smtp.received().is_empty());
}

/// `draft send --at` planifie l'envoi à l'instant donné ; `resolve-send` d'une action inconnue : code 11
#[tokio::test]
async fn draft_send_at_schedules_the_instant() {
    let (bench, account) = FullBench::start_seeded().await;
    let created = bench.run_ok(&["draft", "create", &account]).await;
    let id = created["data"]["data"]["id"].as_str().unwrap().to_string();
    bench
        .run_ok(&["draft", "update", &id, "--to", "bob@example.org"])
        .await;
    bench
        .run_ok(&["draft", "send", &id, "--at", "2030-01-01"])
        .await;
    let outbox = bench.run_ok(&["draft", "outbox", &account]).await;
    assert_eq!(
        outbox["data"]["data"][0]["scheduled_for"],
        1_893_456_000_000_i64
    );
    let unknown = bench
        .run(&["draft", "resolve-send", "inconnue", "--resend"])
        .await;
    assert_eq!(unknown.code, 11, "{}", unknown.stderr);
}
