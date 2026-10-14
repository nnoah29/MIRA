#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Règle d'incertitude d'un envoi : seules les pannes prouvées avant DATA sont passagères.

use mira_core::actions::send_outcome_is_unknown;
use mira_vocabulary::error::codes::{network, server, tls};
use mira_vocabulary::{ErrorCategory, ErrorCode, MiraError};

/// Erreur de test de la catégorie et du code donnés
fn error(category: ErrorCategory, code: ErrorCode, retryable: bool) -> MiraError {
    MiraError::new(category, code, "test").retryable(retryable)
}

/// Délai, coupure et autres pannes de transport après la connexion rendent l'issue incertaine
#[test]
fn regression_transport_failures_after_connection_leave_the_outcome_unknown() {
    for code in [network::TIMEOUT, network::CONNECTION_LOST] {
        let failure = error(ErrorCategory::Network, code, false);
        assert!(send_outcome_is_unknown(&failure), "{code:?}");
    }
    let other = error(ErrorCategory::Network, ErrorCode("network.io"), false);
    assert!(send_outcome_is_unknown(&other));
}

/// Hôte injoignable, refus 4xx et échec TLS sont prouvés avant DATA : rejeu permis
#[test]
fn failures_proven_before_data_do_not_make_the_outcome_unknown() {
    let unreachable = error(ErrorCategory::Network, network::UNREACHABLE, true);
    let greylisted = error(ErrorCategory::Server, server::RATE_LIMITED, true);
    let handshake = error(ErrorCategory::Tls, tls::HANDSHAKE, false);
    for failure in [unreachable, greylisted, handshake] {
        assert!(!send_outcome_is_unknown(&failure), "{failure:?}");
    }
}
