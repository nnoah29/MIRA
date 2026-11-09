#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Contrat 1.9 : un client écrit pour 1.8 parle encore au moteur, un client 1.10 non.

use mira_vocabulary::{CONTRACT_VERSION, ContractVersion};

/// Le moteur 1.9 accepte un client 1.8 et refuse un client 1.10.
#[test]
fn a_1_8_client_can_talk_to_a_1_9_engine_but_not_a_1_10_client() {
    assert_eq!(CONTRACT_VERSION, ContractVersion { major: 1, minor: 9 });
    assert!(CONTRACT_VERSION.is_compatible_with(ContractVersion { major: 1, minor: 8 }));
    assert!(!CONTRACT_VERSION.is_compatible_with(ContractVersion {
        major: 1,
        minor: 10
    }));
}
