//! Moteur de test dont la synchronisation en arrière-plan est active, et courrier à déposer sur le faux serveur.

use std::time::Duration;

use mira_core::sync::{BackgroundSync, BackoffPolicy, SupervisorConfig};

use super::fixture::{self, Fixture};

/// Délais courts : reconnexion entre 50 et 200 ms, relevé périodique très espacé pour que seuls IDLE et les demandes agissent
pub fn quick_sync() -> BackgroundSync {
    BackgroundSync::On(SupervisorConfig {
        backoff: BackoffPolicy {
            initial: Duration::from_millis(50),
            max: Duration::from_millis(200),
        },
        idle_renewal: Duration::from_secs(30),
        poll_interval: Some(Duration::from_secs(600)),
    })
}

/// Moteur qui synchronise en arrière-plan, relié au faux serveur IMAP
pub async fn start_syncing() -> Fixture {
    fixture::start_with(quick_sync()).await
}

/// Courrier avec objet et identifiant de message distincts
pub fn mail(subject: &str, message_id: &str) -> Vec<u8> {
    format!(
        "From: Bob <bob@example.org>\r\nTo: alice@example.org\r\nSubject: {subject}\r\nMessage-ID: <{message_id}>\r\nDate: Mon, 01 Jan 2024 10:00:00 +0000\r\n\r\nCorps de {subject}\r\n"
    )
    .into_bytes()
}

/// Secondes Unix d'il y a `days` jours
pub fn days_ago(days: i64) -> i64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    i64::try_from(now).unwrap() - days * 86_400
}

/// Dépose un message d'objet donné dans la boîte du compte `folders`
pub fn deliver_one(fixture: &Fixture, mailbox: &str, subject: &str) {
    let raw = mail(subject, &format!("{subject}@x"));
    let user = fixture::FOLDERS_USER;
    fixture.store.deliver(user, mailbox, &raw, &[]);
}

/// Dépose `count` messages « <préfixe> <n> » dans la boîte du compte `folders`
pub fn deliver_many(fixture: &Fixture, mailbox: &str, prefix: &str, count: u32) {
    for n in 1..=count {
        let raw = mail(&format!("{prefix} {n}"), &format!("{prefix}{n}@x"));
        fixture
            .store
            .deliver(fixture::FOLDERS_USER, mailbox, &raw, &[]);
    }
}
