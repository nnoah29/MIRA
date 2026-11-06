//! Démon sur de faux serveurs IMAP et SMTP : compte ajouté, synchronisation lancée, boîte de réception prête.

use std::time::Duration;

use mira_assembly::api::{AccountDraft, AccountRequest, CredentialInput, Request, SecretInput};
use mira_assembly::mira_vocabulary::{AuthMethodKind, IdempotencyKey};
use mira_testkit::imap::FakeImapServer;
use mira_testkit::smtp::FakeSmtpServer;
use std::sync::Arc;

use mira_assembly::ports::ManualClock;
use serde_json::{Value, json};

use super::mail_fixture::{PASSWORD, USER, config_with_clock, servers, test_clock};
use super::{TestClient, TestDaemon};

/// Démon dont le moteur parle aux faux serveurs ; les serveurs vivent aussi longtemps que lui.
pub struct MailDaemon {
    pub daemon: TestDaemon,
    pub imap: FakeImapServer,
    pub smtp: FakeSmtpServer,
    pub home: tempfile::TempDir,
    pub clock: Arc<ManualClock>,
}

impl MailDaemon {
    /// Démarre les faux serveurs puis un démon dont le moteur est ouvert dessus.
    pub async fn start() -> Self {
        let (imap, smtp) = servers().await;
        let home = tempfile::tempdir().unwrap();
        let clock = test_clock();
        let engine = config_with_clock(home.path(), imap.port(), smtp.port(), clock.clone());
        let daemon = TestDaemon::start(engine).await;
        Self {
            daemon,
            imap,
            smtp,
            home,
            clock,
        }
    }

    /// Client attaché au démon.
    pub async fn attach(&self) -> TestClient {
        let mut client = TestClient::connect(&self.daemon).await;
        client.hello_ok(self.daemon.token()).await;
        client
    }
}

/// Ajoute le compte de test et rend son identifiant.
pub async fn add_account(client: &mut TestClient) -> String {
    let draft = AccountDraft {
        address: USER.into(),
        display_name: "Alice".into(),
        provider: None,
        servers: None,
        auth: AuthMethodKind::Password,
    };
    let request = Request::Account(AccountRequest::AddAccount {
        draft,
        credential: CredentialInput::Password(SecretInput::new(PASSWORD)),
        idempotency_key: IdempotencyKey("daemon-test".into()),
    });
    let envelope = client.call(serde_json::to_value(request).unwrap()).await;
    assert_eq!(envelope["ok"], true, "{envelope}");
    envelope["data"]["data"]["id"].as_str().unwrap().to_string()
}

/// Lance la synchronisation de tous les comptes.
pub async fn start_sync(client: &mut TestClient) {
    let request =
        json!({"domain": "sync", "request": {"op": "start_sync", "scope": {"type": "all"}}});
    assert_eq!(client.call(request).await["ok"], true);
}

/// Attend que la boîte de réception contienne trois messages et rend la liste `(sujet, identifiant)`.
pub async fn wait_inbox(client: &mut TestClient, account: &str) -> Vec<(String, String)> {
    let boxes =
        json!({"domain": "mailbox", "request": {"op": "list_mailboxes", "account": account}});
    for _ in 0..150 {
        let reply = client.call(boxes.clone()).await;
        let inbox = reply["data"]["data"].as_array().and_then(|list| {
            list.iter()
                .find(|b| b["role"] == "inbox" && b["total"].as_u64() >= Some(3))
        });
        if let Some(inbox) = inbox {
            return list_messages(client, inbox["id"].as_str().unwrap()).await;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("réception jamais synchronisée");
}

/// Liste la première page d'une boîte, comme le fait l'interface Flutter.
async fn list_messages(client: &mut TestClient, mailbox: &str) -> Vec<(String, String)> {
    let filters = json!({"unread_only": false, "flagged_only": false, "has_attachments": false,
        "from_contains": null, "since": null, "until": null});
    let request = json!({"domain": "message", "request": {
        "op": "list_messages", "view": {"type": "mailbox", "mailbox": mailbox},
        "filters": filters, "page": {"cursor": null, "limit": 200}}});
    let page: Value = client.call(request).await;
    let items = page["data"]["data"]["items"].as_array().unwrap();
    let pair = |m: &Value| {
        (
            m["subject"].as_str().unwrap().to_string(),
            m["id"].as_str().unwrap().to_string(),
        )
    };
    items.iter().map(pair).collect()
}
