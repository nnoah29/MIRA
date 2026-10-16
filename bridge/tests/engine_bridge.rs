#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Pont de bout en bout : JSON du contrat → moteur réel → faux serveur IMAP du testkit (127.0.0.1).

use std::sync::Arc;
use std::time::Duration;

use mira_assembly::accounts::NetworkPolicy;
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

/// Configuration de test : tout sous un dossier jetable, le domaine `bridge.test` vise le faux serveur.
fn config(home: &std::path::Path, port: u16) -> EngineConfig {
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
            smtp: endpoint(1),
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
                .find(|b| b["role"] == "inbox" && b["total"].as_u64() > Some(0))
        });
        if let Some(inbox) = inbox {
            let id = inbox["id"].as_str().unwrap().to_string();
            return (id.clone(), list_messages(&id).await);
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("réception jamais synchronisée");
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

/// Démarre un faux serveur garni et ouvre le moteur dessus ; rend ce qui doit rester en vie.
async fn start() -> (tempfile::TempDir, FakeImapServer) {
    let credentials = Credentials::default();
    credentials.add_password(USER, PASSWORD);
    let store = FakeMailStore::new();
    store.deliver(
        USER,
        "INBOX",
        &raw_mail("Bonjour", "Texte du premier."),
        &[],
    );
    store.deliver(
        USER,
        "INBOX",
        &raw_mail("Second", "Texte du second."),
        &["\\Seen"],
    );
    let imap = FakeImapServer::start(store, credentials, CapabilitySet::full())
        .await
        .unwrap();
    let home = tempfile::tempdir().unwrap();
    let opened: Value =
        serde_json::from_str(&runtime::open(config(home.path(), imap.port()))).unwrap();
    assert_eq!(opened["ok"], true, "{opened}");
    (home, imap)
}

/// Comptes, boîtes, messages et corps arrivent en JSON du contrat, sans secret.
#[tokio::test(flavor = "multi_thread")]
async fn the_bridge_serves_real_engine_values_as_json() {
    let (_home, _imap) = start().await;
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
    assert_eq!(subjects, ["Bonjour", "Second"]);
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
    assert!(runtime::close().await.contains("\"ok\":true"));
}

/// Une requête illisible ou un moteur fermé donnent une enveloppe d'erreur, jamais un plantage.
#[tokio::test(flavor = "multi_thread")]
async fn the_bridge_reports_errors_as_envelopes() {
    let closed: Value = serde_json::from_str(&runtime::execute("{}").await).unwrap();
    assert_eq!(closed["ok"], false);
    assert_eq!(closed["contract_version"]["major"], 1);
}
