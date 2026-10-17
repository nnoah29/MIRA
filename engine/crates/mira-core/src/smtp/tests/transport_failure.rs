//! Classement des pannes de transport : seul un échec de connexion TCP est passager.

use mira_vocabulary::ErrorCategory;

use crate::smtp::errors::transport_failure;

/// Une panne dans la session (délai ETIMEDOUT compris) est incertaine : ni réessayable ni `network.timeout`
#[test]
fn regression_failure_inside_the_session_is_connection_lost() {
    let error = transport_failure(false);
    assert_eq!(error.category, ErrorCategory::Network);
    assert_eq!(error.code, "network.connection_lost");
    assert!(!error.retryable);
}

/// Un échec avant la session (connexion TCP, délai de connexion compris) est passager
#[test]
fn failure_before_the_session_is_unreachable_and_retryable() {
    let error = transport_failure(true);
    assert_eq!(error.code, "network.unreachable");
    assert!(error.retryable);
}
