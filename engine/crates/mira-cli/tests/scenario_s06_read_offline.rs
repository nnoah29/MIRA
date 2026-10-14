#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! S6 par le CLI : lecture sans connexion, corps présent, en-tête seul, pièce jointe non téléchargée.

#[allow(dead_code)]
mod cli_support;

use cli_support::parse_json;
use cli_support::sa_s6_reading::start_read_world;

/// Corps préchargé : `message body` le rend tout de suite hors ligne, sans ouvrir de connexion (inv. 27)
#[tokio::test]
async fn scenario_s6_present_body_is_read_offline_via_cli() {
    let mut fx = start_read_world().await;
    let connections = fx.world.bench.imap.connection_count();
    fx.world.go_offline();
    let body = fx.world.run_ok(&["message", "body", &fx.present]).await;
    assert!(
        body["data"]["text"]
            .as_str()
            .unwrap()
            .contains("Corps de Alpha")
    );
    let human = fx.world.run(&["message", "body", &fx.present]).await;
    assert_eq!(human.code, 0, "{}", human.stderr);
    assert!(human.stdout.starts_with("Corps de Alpha"));
    let got = fx.world.run_ok(&["message", "get", &fx.present]).await;
    assert_eq!(got["data"]["content"], "body_present");
    assert_eq!(fx.world.bench.imap.connection_count(), connections);
}

/// En-tête seul : code 12 non réessayable hors ligne ; `message fetch` échoue en 13 puis, réseau revenu, le corps se lit hors ligne
#[tokio::test]
async fn scenario_s6_header_only_body_is_unavailable_until_fetched_via_cli() {
    let mut fx = start_read_world().await;
    fx.world.go_offline();
    let args = ["message", "body", &fx.header_only, "--json"];
    let refused = fx.world.run(&args).await;
    assert_eq!(refused.code, 12, "{}", refused.stderr);
    let error = &parse_json(&refused)["error"];
    assert_eq!(error["code"], "incompatible_state.body_unavailable_offline");
    assert_eq!(error["retryable"], false);
    let got = fx.world.run_ok(&["message", "get", &fx.header_only]).await;
    assert_eq!(got["data"]["content"], "headers_only");
    let fetch = ["message", "fetch", &fx.header_only, "--json"];
    let offline = fx.world.run(&fetch).await;
    assert_eq!(offline.code, 13, "{}", offline.stdout);
    assert_eq!(parse_json(&offline)["error"]["retryable"], true);
    fx.world.go_online();
    fx.world
        .run_ok(&["message", "fetch", &fx.header_only])
        .await;
    fx.world.go_offline();
    let body = fx.world.run_ok(&["message", "body", &fx.header_only]).await;
    assert!(
        body["data"]["text"]
            .as_str()
            .unwrap()
            .contains("Corps de Range")
    );
}

/// Message en-tête seul : aucune pièce jointe connue hors ligne ; message préchargé : pièce enregistrée depuis la source locale
#[tokio::test]
async fn scenario_s6_attachments_follow_the_local_source_offline_via_cli() {
    let mut fx = start_read_world().await;
    let attachment = fx.attachment_id().await;
    fx.world.go_offline();
    assert!(
        fx.attachments_of(&fx.notes)
            .await
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        fx.attachments_of(&fx.with_file).await[0]["cache"],
        "not_downloaded"
    );
    let target = tempfile::tempdir().unwrap();
    let file = target.path().join("data.bin");
    let path = file.to_str().unwrap();
    fx.world
        .run_ok(&["message", "save-attachment", &attachment, path])
        .await;
    let size = cli_support::raw_mail::ATTACHMENT_SIZE;
    assert_eq!(std::fs::read(&file).unwrap().len(), size);
    assert_eq!(fx.attachments_of(&fx.with_file).await[0]["cache"], "cached");
}

/// Listes, recherche locale, fils et compteurs : même réponse serveur coupé, et aucune connexion ouverte (inv. 27)
#[tokio::test]
async fn scenario_s6_lists_and_search_work_with_the_server_down_via_cli() {
    let fx = start_read_world().await;
    let reads: [&[&str]; 4] = [
        &["message", "list", "--mailbox", &fx.inbox],
        &["mailbox", "list", &fx.account],
        &["search", "local", "Alpha"],
        &["thread", "list", &fx.inbox],
    ];
    let mut before = Vec::new();
    for line in reads {
        before.push(fx.world.run_ok(line).await);
    }
    let connections = fx.world.bench.imap.connection_count();
    fx.world.bench.imap.shutdown();
    for (line, expected) in reads.iter().zip(&before) {
        assert_eq!(&fx.world.run_ok(line).await, expected, "{line:?}");
    }
    assert_eq!(fx.world.bench.imap.connection_count(), connections);
}

/// Aucune lecture hors ligne ne fait apparaître le mot de passe, ni dans les sorties ni dans le profil
#[tokio::test]
async fn scenario_s6_offline_reads_never_leak_the_password_via_cli() {
    let mut fx = start_read_world().await;
    fx.world.go_offline();
    fx.world.run_ok(&["message", "body", &fx.present]).await;
    fx.world.run(&["message", "body", &fx.header_only]).await;
    fx.world.run(&["message", "raw", &fx.present]).await;
    fx.world
        .assert_no_secret_anywhere(&[fx.world.password.as_str()]);
}
