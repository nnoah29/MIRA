#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Contrat 1.12 : un client écrit pour 1.11 parle encore au moteur, un client 1.13 non.

use mira_vocabulary::{CONTRACT_VERSION, ContractVersion};

/// Le moteur 1.12 accepte un client 1.11 et refuse un client 1.13.
#[test]
fn a_1_11_client_can_talk_to_a_1_12_engine_but_not_a_1_13_client() {
    assert_eq!(
        CONTRACT_VERSION,
        ContractVersion {
            major: 1,
            minor: 12
        }
    );
    assert!(CONTRACT_VERSION.is_compatible_with(ContractVersion {
        major: 1,
        minor: 11
    }));
    assert!(!CONTRACT_VERSION.is_compatible_with(ContractVersion {
        major: 1,
        minor: 13
    }));
}
