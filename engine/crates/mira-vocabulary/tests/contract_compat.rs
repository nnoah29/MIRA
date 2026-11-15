#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Contrat 1.11 : un client écrit pour 1.10 parle encore au moteur, un client 1.12 non.

use mira_vocabulary::{CONTRACT_VERSION, ContractVersion};

/// Le moteur 1.11 accepte un client 1.10 et refuse un client 1.12.
#[test]
fn a_1_10_client_can_talk_to_a_1_11_engine_but_not_a_1_12_client() {
    assert_eq!(
        CONTRACT_VERSION,
        ContractVersion {
            major: 1,
            minor: 11
        }
    );
    assert!(CONTRACT_VERSION.is_compatible_with(ContractVersion {
        major: 1,
        minor: 10
    }));
    assert!(!CONTRACT_VERSION.is_compatible_with(ContractVersion {
        major: 1,
        minor: 12
    }));
}
