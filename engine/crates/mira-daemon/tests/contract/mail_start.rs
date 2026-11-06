//! Démarrage d'un moteur sur de faux serveurs, compte de test et attente de la réception.

use std::time::Duration;

use mira_assembly::api::{AccountDraft, AccountRequest, CredentialInput, Request, SecretInput};
use mira_assembly::mira_vocabulary::{AuthMethodKind, IdempotencyKey};
use mira_testkit::imap::FakeImapServer;
use mira_testkit::smtp::FakeSmtpServer;
use serde_json::{Value, json};

use super::driver::Driver;
use super::mail_fixture::{PASSWORD, USER, config, servers};

/// Moteur ouvert sur les faux serveurs ; tout ce qui doit rester en vie y est rangé.
pub struct Started<D> {
    pub driver: D,
    pub home: tempfile::TempDir,
    pub imap: FakeImapServer,
    pub smtp: FakeSmtpServer,
}

/// Démarre les faux serveurs (IMAP garni, SMTP) et ouvre le moteur dessus par ce chemin.
pub async fn start<D: Driver>() -> Started<D> {
    let (imap, smtp) = servers().await;
    let home = tempfile::tempdir().unwrap();
    let driver = D::start(Some(config(home.path(), imap.port(), smtp.port()))).await;
    Started {
        driver,
        home,
        imap,
        smtp,
    }
}

/// Envoie une requête déjà en JSON et rend l'enveloppe décodée.
pub async fn send<D: Driver>(driver: &D, request: &Value) -> Value {
    serde_json::from_str(&driver.execute(&request.to_string()).await).unwrap()
}

/// Ajoute le compte de test et rend son identifiant.
pub async fn add_account<D: Driver>(driver: &D) -> String {
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
        idempotency_key: IdempotencyKey("bridge-test".into()),
    });
    let envelope = send(driver, &serde_json::to_value(request).unwrap()).await;
    assert_eq!(envelope["ok"], true, "{envelope}");
    envelope["data"]["data"]["id"].as_str().unwrap().to_string()
}

/// Liste la première page d'une boîte, comme le fait l'interface Flutter.
pub async fn list_messages<D: Driver>(driver: &D, mailbox: &str) -> Value {
    send(
        driver,
        &json!({"domain": "message", "request": {
            "op": "list_messages",
            "view": {"type": "mailbox", "mailbox": mailbox},
            "filters": {"unread_only": false, "flagged_only": false, "has_attachments": false,
                        "from_contains": null, "since": null, "until": null},
            "page": {"cursor": null, "limit": 200}
        }}),
    )
    .await
}

/// Attend que la première boîte de réception contienne des messages et rend la liste JSON.
pub async fn wait_inbox<D: Driver>(driver: &D, account: &str) -> (String, Value) {
    let request =
        json!({"domain": "mailbox", "request": {"op": "list_mailboxes", "account": account}});
    for _ in 0..150 {
        let boxes = send(driver, &request).await;
        let inbox = boxes["data"]["data"].as_array().and_then(|list| {
            list.iter()
                .find(|b| b["role"] == "inbox" && b["total"].as_u64() >= Some(3))
        });
        if let Some(inbox) = inbox {
            let id = inbox["id"].as_str().unwrap().to_string();
            return (id.clone(), list_messages(driver, &id).await);
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let last = send(driver, &request).await;
    panic!("réception jamais synchronisée : {last}");
}
