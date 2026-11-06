#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Aucune trame émise par le démon ne contient un secret : mot de passe, clé d'IA ou jeton local.

mod support;

use serde_json::json;
use support::ai_daemon::AiDaemon;
use support::chat_fixture::chat_json;
use support::mail_daemon::{MailDaemon, add_account, start_sync, wait_inbox};
use support::mail_fixture::PASSWORD;

const AI_KEY: &str = "sk-BRIDGE-SECRET-KEY";

/// Les trames d'un scénario courrier complet ne contiennent ni le mot de passe du compte ni le jeton.
#[tokio::test(flavor = "multi_thread")]
async fn no_frame_from_the_daemon_contains_a_mail_secret() {
    let mail = MailDaemon::start().await;
    let mut client = mail.attach().await;
    let account = add_account(&mut client).await;
    start_sync(&mut client).await;
    wait_inbox(&mut client, &account).await;
    let accounts = json!({"domain": "account", "request": {"op": "list_accounts"}});
    assert_eq!(client.call(accounts).await["ok"], true);
    let seen = String::from_utf8_lossy(client.received_bytes()).to_string();
    assert!(
        seen.len() > 1000,
        "le scénario doit avoir échangé des trames"
    );
    assert!(!seen.contains(PASSWORD), "mot de passe dans une trame");
    assert!(!seen.contains(mail.daemon.token()), "jeton dans une trame");
}

/// Les trames d'une configuration d'IA et d'un chat ne contiennent ni la clé ni le jeton ; `has_key` seul le dit.
#[tokio::test(flavor = "multi_thread")]
async fn no_frame_from_the_daemon_contains_an_ai_secret() {
    let ai = AiDaemon::start().await;
    let mut client = ai.attach().await;
    let list = json!({"domain": "ai", "request": {"op": "list_ai_connections"}});
    let listed = client.call(list).await;
    assert!(listed.to_string().contains("\"has_key\":true"), "{listed}");
    ai.server
        .queue(mira_testkit::ai::openai_stream(&["Bonjour."], None));
    client
        .open_chat(5, &chat_json(Some(&ai.connection), "Salut"), 16)
        .await;
    let (_, reason) = client.collect_until_end(5).await;
    assert_eq!(reason, mira_protocol::EndReason::Finished);
    let mut seen = String::from_utf8_lossy(&ai.setup_bytes).to_string();
    seen.push_str(&String::from_utf8_lossy(client.received_bytes()));
    assert!(!seen.contains(AI_KEY), "clé d'IA dans une trame");
    assert!(!seen.contains(ai.daemon.token()), "jeton dans une trame");
}
