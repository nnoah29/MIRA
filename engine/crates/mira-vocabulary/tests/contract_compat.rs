#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Contrat 1.7 : un client écrit pour 1.6 parle encore au moteur, un client 1.8 non.

use mira_vocabulary::{CONTRACT_VERSION, ContractVersion};

/// Le moteur 1.7 accepte un client 1.6 et refuse un client 1.8.
#[test]
fn a_1_6_client_can_talk_to_a_1_7_engine_but_not_a_1_8_client() {
    assert_eq!(CONTRACT_VERSION, ContractVersion { major: 1, minor: 7 });
    assert!(CONTRACT_VERSION.is_compatible_with(ContractVersion { major: 1, minor: 6 }));
    assert!(!CONTRACT_VERSION.is_compatible_with(ContractVersion { major: 1, minor: 8 }));
}
