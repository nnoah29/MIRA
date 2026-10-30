#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use mira_vocabulary::{CONTRACT_VERSION, ContractVersion, PageRequest};

/// Invariant 33 : même majeure et mineure client au plus égale à celle du moteur.
#[test]
fn invariant_33_contract_version_compatibility() {
    let engine = ContractVersion { major: 1, minor: 2 };
    assert!(engine.is_compatible_with(ContractVersion { major: 1, minor: 0 }));
    assert!(engine.is_compatible_with(ContractVersion { major: 1, minor: 2 }));
    assert!(!engine.is_compatible_with(ContractVersion { major: 1, minor: 3 }));
    assert!(!engine.is_compatible_with(ContractVersion { major: 2, minor: 0 }));
    assert!(!engine.is_compatible_with(ContractVersion { major: 0, minor: 0 }));
}

/// La version courante du contrat est 1.2 et se sérialise en objet.
#[test]
fn current_contract_version_is_one_dot_zero() {
    assert_eq!(CONTRACT_VERSION, ContractVersion { major: 1, minor: 2 });
    let json = serde_json::to_string(&CONTRACT_VERSION).unwrap();
    assert_eq!(json, r#"{"major":1,"minor":2}"#);
}

/// La taille de page demandée est ramenée entre 1 et 500 (invariant 29).
#[test]
fn page_request_limit_is_bounded() {
    let request = |limit| PageRequest {
        cursor: None,
        limit,
    };
    assert_eq!(PageRequest::default().limit, 50);
    assert_eq!(request(0).bounded_limit(), 1);
    assert_eq!(request(50).bounded_limit(), 50);
    assert_eq!(request(500).bounded_limit(), 500);
    assert_eq!(request(10_000).bounded_limit(), 500);
}
