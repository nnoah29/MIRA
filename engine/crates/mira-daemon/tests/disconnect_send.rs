#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Une déconnexion n'annule jamais un envoi ni une synchronisation : ils appartiennent au moteur.

mod support;

use std::time::Duration;

use mira_assembly::api::{ComposeRequest, DraftChanges, DraftMode};
use mira_assembly::mira_vocabulary::{AccountId, DraftId};
use serde_json::{Value, json};
use support::TestClient;
use support::mail_daemon::{MailDaemon, add_account, wait_inbox};

/// Crée un brouillon prêt pour Bob et rend son identifiant.
async fn ready_draft(client: &mut TestClient, account: &str) -> String {
    let create = ComposeRequest::CreateDraft {
        account: AccountId::from_raw(account),
        identity: None,
        mode: DraftMode::New,
    };
    let created = client.call(request_of(create)).await;
    let id = created["data"]["data"]["data"]["id"]
        .as_str()
        .unwrap_or_else(|| panic!("{created}"))
        .to_string();
    let changes: DraftChanges = serde_json::from_value(json!({
        "to": [{"address": "bob@example.org", "name": null}],
        "subject": "Bonjour Bob", "body_text": "Corps du message"
    }))
    .unwrap();
    let update = ComposeRequest::UpdateDraft {
        draft: DraftId::from_raw(id.clone()),
        changes,
    };
    assert_eq!(client.call(request_of(update)).await["ok"], true);
    id
}

/// Enveloppe une requête de composition pour le contrat.
fn request_of(request: ComposeRequest) -> Value {
    serde_json::to_value(mira_assembly::api::Request::Compose(request)).unwrap()
}

/// Un envoi dont l'horloge arrive à échéance après la déconnexion du client aboutit.
#[tokio::test(flavor = "multi_thread")]
async fn a_message_send_survives_the_client_disconnecting() {
    let mail = MailDaemon::start().await;
    let mut client = mail.attach().await;
    let account = add_account(&mut client).await;
    let draft = ready_draft(&mut client, &account).await;
    let send = json!({"domain": "compose", "request": {"op": "send_draft", "draft": draft, "at": null, "idempotency_key": "k1"}});
    assert_eq!(client.call(send).await["ok"], true);
    drop(client);
    mail.clock.advance_millis(10_001);
    let limit = tokio::time::Instant::now() + Duration::from_secs(15);
    while mail.smtp.received().is_empty() && tokio::time::Instant::now() < limit {
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert_eq!(mail.smtp.received().len(), 1);
    assert_eq!(mail.smtp.received()[0].recipients, ["bob@example.org"]);
}

/// Une synchronisation lancée par un client qui se déconnecte aussitôt va jusqu'au bout.
#[tokio::test(flavor = "multi_thread")]
async fn a_sync_started_by_a_client_continues_after_it_disconnects() {
    let mail = MailDaemon::start().await;
    let mut client = mail.attach().await;
    let account = add_account(&mut client).await;
    let sync = json!({"domain": "sync", "request": {"op": "start_sync", "scope": {"type": "all"}}});
    client.send_call(sync).await;
    drop(client);
    let mut other = mail.attach().await;
    let subjects: Vec<String> = wait_inbox(&mut other, &account)
        .await
        .into_iter()
        .map(|m| m.0)
        .collect();
    assert_eq!(subjects.len(), 3, "{subjects:?}");
}
