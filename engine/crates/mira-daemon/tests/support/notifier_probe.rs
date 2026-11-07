//! Outils des tests du notifieur : compte prêt et synchronisé, courrier à livrer, attente des notifications.

use std::time::Duration;

use super::mail_daemon::{MailDaemon, add_account, start_sync, wait_inbox};
use super::mail_fixture::USER;
use super::notifier_sink::CollectingSink;
use super::{TestClient, mail_samples};

/// Démon sur de faux serveurs, compte ajouté et déjà synchronisé ; rend aussi un client attaché et l'identifiant du compte.
pub async fn synced_daemon() -> (MailDaemon, TestClient, String) {
    let daemon = MailDaemon::start().await;
    let mut client = daemon.attach().await;
    let account = add_account(&mut client).await;
    start_sync(&mut client).await;
    wait_inbox(&mut client, &account).await;
    (daemon, client, account)
}

/// Dépose un message d'objet donné dans la réception du faux serveur puis demande une synchronisation.
pub async fn deliver(daemon: &MailDaemon, client: &mut TestClient, subject: &str) {
    let raw = mail_samples::raw_mail(subject, "Corps du message.");
    daemon.imap.state().deliver(USER, "INBOX", &raw, &[]);
    start_sync(client).await;
}

/// Attend, au plus `wait`, que le récepteur ait reçu au moins `count` notifications.
pub async fn wait_for(sink: &CollectingSink, count: usize, wait: Duration) -> bool {
    let deadline = tokio::time::Instant::now() + wait;
    while tokio::time::Instant::now() < deadline {
        if sink.shown().len() >= count {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    false
}
