#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Correspondance entre erreurs du serveur et alertes annoncées.

use mira_core::events::alert_for;
use mira_vocabulary::error::codes::{not_found, server, tls};
use mira_vocabulary::{AccountId, ErrorCategory, ErrorCode, EventKind, MiraError};

/// Erreur de test portant ce code
fn failing(code: ErrorCode) -> MiraError {
    MiraError::new(ErrorCategory::Server, code, "échec de test")
}

/// Quota dépassé et débit limité ont leur alerte, pour le compte concerné
#[test]
fn quota_and_rate_limit_have_their_alert() {
    let account = AccountId::from_raw("a1");
    let quota = alert_for(&account, &failing(server::QUOTA_EXCEEDED));
    let rate = alert_for(&account, &failing(server::RATE_LIMITED));
    assert_eq!(
        quota,
        Some(EventKind::QuotaFull {
            account: account.clone()
        })
    );
    assert_eq!(rate, Some(EventKind::RateLimited { account }));
}

/// Les autres erreurs, dont le certificat, n'ont pas d'alerte
#[test]
fn other_errors_have_no_alert() {
    let account = AccountId::from_raw("a1");
    for code in [
        server::REJECTED,
        tls::INVALID_CERTIFICATE,
        not_found::MESSAGE,
    ] {
        assert_eq!(alert_for(&account, &failing(code)), None);
    }
}
