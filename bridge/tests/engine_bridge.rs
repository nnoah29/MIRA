#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Pont de bout en bout : JSON du contrat → moteur réel → faux serveur IMAP du testkit (127.0.0.1).

use std::sync::Arc;
use std::time::Duration;

use mira_assembly::accounts::NetworkPolicy;
use mira_assembly::ai::AiProviderRegistry;
use mira_assembly::api::{
    AccountDraft, AccountRequest, CredentialInput, EngineConfig, Request, SecretInput,
};
use mira_assembly::auth::OAuthClientConfig;
use mira_assembly::build_registry;
use mira_assembly::mira_vocabulary::{
    AuthMethodKind, IdempotencyKey, ProfileId, Security, ServerEndpoint, ServerSettings, Timestamp,
};
use mira_assembly::ports::{ManualClock, MemorySecretStore, SequentialIds};
use mira_assembly::profile::ProfileEnvironment;
use mira_assembly::sync::{BackgroundSync, SupervisorConfig};
use mira_bridge::runtime;
use mira_testkit::Credentials;
use mira_testkit::imap::{CapabilitySet, FakeImapServer, FakeMailStore};
use mira_testkit::smtp::FakeSmtpServer;
use serde_json::{Value, json};

const USER: &str = "alice@bridge.test";
const PASSWORD: &str = "pa55-SECRET-xyz";

/// Message brut minimal, expédié par Bob.
fn raw_mail(subject: &str, body: &str) -> Vec<u8> {
    format!(
        "From: Bob <bob@example.org>\r\nTo: Alice <{USER}>\r\nSubject: {subject}\r\n\
         Date: Mon, 01 Jan 2024 10:00:00 +0000\r\nMessage-ID: <{subject}@example.org>\r\n\
         Content-Type: text/plain; charset=utf-8\r\n\r\n{body}\r\n"
    )
    .into_bytes()
}

/// Message HTML avec une image incorporée (`cid:`) et une image distante.
fn html_mail() -> Vec<u8> {
    format!(
        "From: Bob <bob@example.org>\r\nTo: {USER}\r\nSubject: Riche\r\n\
         Date: Mon, 01 Jan 2024 11:00:00 +0000\r\nMessage-ID: <riche@example.org>\r\n\
         MIME-Version: 1.0\r\nContent-Type: multipart/related; boundary=\"B\"\r\n\r\n\
         --B\r\nContent-Type: text/html; charset=utf-8\r\n\r\n\
         <table><tr><td>Bonjour</td></tr></table><img src=\"cid:logo@x\">\
         <img src=\"https://tracker.example/p.png\">\r\n\
         --B\r\nContent-Type: image/png\r\nContent-ID: <logo@x>\r\n\
         Content-Disposition: inline; filename=\"logo.png\"\r\n\
         Content-Transfer-Encoding: base64\r\n\r\niVBORw0KGgo=\r\n--B--\r\n"
    )
    .into_bytes()
}

/// Configuration de test : tout sous un dossier jetable, le domaine `bridge.test` vise les faux serveurs.
fn config(home: &std::path::Path, port: u16, smtp_port: u16) -> EngineConfig {
    let endpoint = |port| ServerEndpoint {
        host: "127.0.0.1".into(),
        port,
        security: Security::Plaintext,
    };
    let mut network = NetworkPolicy::default();
    network.server_overrides.insert(
        "bridge.test".into(),
        ServerSettings {
            imap: endpoint(port),
            smtp: endpoint(smtp_port),
        },
    );
    network.background_sync = BackgroundSync::On(SupervisorConfig::default());
    EngineConfig {
        profile: ProfileId::from_raw("default"),
        environment: ProfileEnvironment {
            mira_home: Some(home.to_path_buf()),
            xdg_data_home: None,
            home: None,
        },
        registry: build_registry(),
        ai_registry: AiProviderRegistry::new(),
        secrets: Arc::new(MemorySecretStore::new()),
        clock: Arc::new(ManualClock::new(Timestamp::from_millis(1_704_100_000_000))),
        ids: Arc::new(SequentialIds::new()),
        oauth: OAuthClientConfig::default(),
        network,
    }
}

/// Exécute une requête et rend l'enveloppe décodée.
async fn run(request: &Request) -> Value {
    let text = runtime::execute(&serde_json::to_string(request).unwrap()).await;
    serde_json::from_str(&text).unwrap()
}

/// Ajoute le compte de test et rend son identifiant.
async fn add_account() -> String {
    let draft = AccountDraft {
        address: USER.into(),
        display_name: "Alice".into(),
        provider: None,
        servers: None,
        auth: AuthMethodKind::Password,
    };
    let envelope = run(&Request::Account(AccountRequest::AddAccount {
        draft,
        credential: CredentialInput::Password(SecretInput::new(PASSWORD)),
        idempotency_key: IdempotencyKey("bridge-test".into()),
    }))
    .await;
    assert_eq!(envelope["ok"], true, "{envelope}");
    envelope["data"]["data"]["id"].as_str().unwrap().to_string()
}

/// Attend que la première boîte de réception contienne des messages et rend la liste JSON.
async fn wait_inbox(account: &str) -> (String, Value) {
    let request =
        json!({"domain": "mailbox", "request": {"op": "list_mailboxes", "account": account}});
    for _ in 0..150 {
        let boxes = send(&request).await;
        let inbox = boxes["data"]["data"].as_array().and_then(|list| {
            list.iter()
                .find(|b| b["role"] == "inbox" && b["total"].as_u64() >= Some(3))
        });
        if let Some(inbox) = inbox {
            let id = inbox["id"].as_str().unwrap().to_string();
            return (id.clone(), list_messages(&id).await);
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let last = send(&request).await;
    panic!("réception jamais synchronisée : {last}");
}

/// Envoie une requête déjà en JSON.
async fn send(request: &Value) -> Value {
    serde_json::from_str(&runtime::execute(&request.to_string()).await).unwrap()
}

/// Liste la première page d'une boîte, comme le fait l'interface Flutter.
async fn list_messages(mailbox: &str) -> Value {
    send(&json!({"domain": "message", "request": {
        "op": "list_messages",
        "view": {"type": "mailbox", "mailbox": mailbox},
        "filters": {"unread_only": false, "flagged_only": false, "has_attachments": false,
                    "from_contains": null, "since": null, "until": null},
        "page": {"cursor": null, "limit": 200}
    }}))
    .await
}

/// Démarre les faux serveurs (IMAP garni, SMTP) et ouvre le moteur dessus ; rend ce qui doit rester en vie.
async fn start() -> (tempfile::TempDir, FakeImapServer, FakeSmtpServer) {
    let credentials = Credentials::default();
    credentials.add_password(USER, PASSWORD);
    let store = FakeMailStore::new();
    store.deliver(
        USER,
        "INBOX",
        &raw_mail("Bonjour", "Texte du premier."),
        &[],
    );
    store.deliver(USER, "INBOX", &html_mail(), &[]);
    store.deliver(
        USER,
        "INBOX",
        &raw_mail("Second", "Texte du second."),
        &["\\Seen"],
    );
    let imap = FakeImapServer::start(store, credentials.clone(), CapabilitySet::full())
        .await
        .unwrap();
    let smtp = FakeSmtpServer::start(credentials).await.unwrap();
    let home = tempfile::tempdir().unwrap();
    let opened: Value = serde_json::from_str(&runtime::open(config(
        home.path(),
        imap.port(),
        smtp.port(),
    )))
    .unwrap();
    assert_eq!(opened["ok"], true, "{opened}");
    (home, imap, smtp)
}

/// Comptes, boîtes, messages et corps arrivent en JSON du contrat, sans secret.
#[tokio::test(flavor = "multi_thread")]
async fn the_bridge_serves_real_engine_values_as_json() {
    let (_home, _imap, _smtp) = start().await;
    let account = add_account().await;
    let started =
        send(&json!({"domain": "sync", "request": {"op": "start_sync", "scope": {"type": "all"}}}))
            .await;
    assert_eq!(started["ok"], true, "{started}");
    let (_, page) = wait_inbox(&account).await;
    let items = page["data"]["data"]["items"].as_array().unwrap();
    let mut subjects: Vec<&str> = items
        .iter()
        .map(|m| m["subject"].as_str().unwrap())
        .collect();
    subjects.sort();
    assert_eq!(subjects, ["Bonjour", "Riche", "Second"]);
    let first = items.iter().find(|m| m["subject"] == "Bonjour").unwrap();
    assert_eq!(first["from"][0]["address"], "bob@example.org");
    assert_eq!(first["flags"]["seen"], false);
    let body = send(&json!({"domain": "message", "request": {"op": "get_message_body", "message": first["id"]}})).await;
    assert!(
        body["data"]["data"]["text"]
            .as_str()
            .unwrap()
            .contains("Texte du premier.")
    );
    assert!(!page.to_string().contains(PASSWORD));
    let rich = items.iter().find(|m| m["subject"] == "Riche").unwrap();
    check_rich_message(&rich["id"]).await;
    check_original_html(rich, first).await;
    check_no_event_replay().await;
    assert!(runtime::close().await.contains("\"ok\":true"));
}

/// Une requête illisible ou un moteur fermé donnent une enveloppe d'erreur, jamais un plantage.
#[tokio::test(flavor = "multi_thread")]
async fn the_bridge_reports_errors_as_envelopes() {
    let closed: Value = serde_json::from_str(&runtime::execute("{}").await).unwrap();
    assert_eq!(closed["ok"], false);
    assert_eq!(closed["contract_version"]["major"], 1);
}

/// Sans numéro de départ, le journal passé n'est pas rejoué ; avec 0, il l'est.
async fn check_no_event_replay() {
    let collect = |after: Option<u64>| async move {
        let seen = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counter = std::sync::Arc::clone(&seen);
        let pump = runtime::pump_events(after, move |_| {
            counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            true
        });
        let _ = tokio::time::timeout(Duration::from_millis(600), pump).await;
        seen.load(std::sync::atomic::Ordering::Relaxed)
    };
    assert_eq!(
        collect(None).await,
        0,
        "le journal passé ne doit pas être rejoué"
    );
    assert!(
        collect(Some(0)).await > 0,
        "depuis 0, le journal est rejoué"
    );
}

/// Le HTML d'origine garde tout (aucun assainissement) et un texte seul n'en donne pas.
async fn check_original_html(rich: &Value, plain: &Value) {
    let original: Value = serde_json::from_str(
        &mira_bridge::original_html::fetch(rich["id"].as_str().unwrap()).await,
    )
    .unwrap();
    let html = original["data"]["html"].as_str().unwrap();
    assert!(html.contains("<table>") && html.contains("cid:logo@x"));
    assert!(html.contains("https://tracker.example/p.png"));
    let none: Value = serde_json::from_str(
        &mira_bridge::original_html::fetch(plain["id"].as_str().unwrap()).await,
    )
    .unwrap();
    assert!(none["data"]["html"].is_null());
}

/// HTML assaini (cid gardé, image distante retirée et signalée) et image incorporée enregistrable.
async fn check_rich_message(id: &Value) {
    let html =
        send(&json!({"domain": "message", "request": {"op": "get_safe_html", "message": id}}))
            .await;
    let safe = &html["data"]["data"];
    assert!(safe["html"].as_str().unwrap().contains("cid:logo@x"));
    assert!(!safe["html"].as_str().unwrap().contains("tracker.example"));
    assert_eq!(
        safe["blocked_resources"][0],
        "https://tracker.example/p.png"
    );
    let list =
        send(&json!({"domain": "message", "request": {"op": "list_attachments", "message": id}}))
            .await;
    let logo = &list["data"]["data"][0];
    assert_eq!(logo["content_id"], "logo@x");
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("logo.png");
    let saved = send(
        &json!({"domain": "message", "request": {"op": "save_attachment",
        "attachment": logo["id"], "destination": target.to_str().unwrap()}}),
    )
    .await;
    assert_eq!(saved["ok"], true, "{saved}");
    for _ in 0..100 {
        if target.exists() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("image incorporée jamais enregistrée");
}
