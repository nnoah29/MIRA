//! Alertes serveur : un échec qui demande l'attention de l'utilisateur est annoncé en plus de l'événement d'échec.

use mira_vocabulary::error::codes::server;
use mira_vocabulary::{AccountId, EventKind, MiraError};

use super::sink::EventSink;

/// Alerte qui correspond à l'erreur : boîte pleine ou débit limité par le serveur
pub fn alert_for(account: &AccountId, error: &MiraError) -> Option<EventKind> {
    let account = account.clone();
    match error.code.as_str() {
        code if code == server::QUOTA_EXCEEDED.0 => Some(EventKind::QuotaFull { account }),
        code if code == server::RATE_LIMITED.0 => Some(EventKind::RateLimited { account }),
        _ => None,
    }
}

/// Annonce l'alerte de l'erreur, s'il y en a une ; l'échec de l'annonce n'ajoute rien à l'erreur d'origine
pub fn announce_alert(sink: &dyn EventSink, account: &AccountId, error: &MiraError) {
    if let Some(alert) = alert_for(account, error) {
        let _ = sink.emit(alert);
    }
}
