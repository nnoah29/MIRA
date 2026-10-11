#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use mira_vocabulary::{CONTRACT_VERSION, ContractVersion};

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

/// La version courante du contrat est 1.0 et se sérialise en objet.
#[test]
fn current_contract_version_is_one_dot_zero() {
    assert_eq!(CONTRACT_VERSION, ContractVersion { major: 1, minor: 0 });
    let json = serde_json::to_string(&CONTRACT_VERSION).unwrap();
    assert_eq!(json, r#"{"major":1,"minor":0}"#);
}
