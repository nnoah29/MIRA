#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Règles de poignée de main : version du protocole d'abord, puis compatibilité du contrat.

use mira_protocol::{HelloRefusal, PROTOCOL_VERSION, ProtocolVersion, check_hello};
use mira_vocabulary::{CONTRACT_VERSION, ContractVersion};

/// Version du contrat avec ces deux nombres.
fn contract(major: u32, minor: u32) -> ContractVersion {
    ContractVersion { major, minor }
}

/// Même majeure et mineure client inférieure ou égale : accepté.
#[test]
fn same_major_and_lower_or_equal_minor_is_accepted() {
    let server = contract(1, 6);
    assert_eq!(
        check_hello(PROTOCOL_VERSION, contract(1, 6), server),
        Ok(())
    );
    assert_eq!(
        check_hello(PROTOCOL_VERSION, contract(1, 0), server),
        Ok(())
    );
    assert_eq!(
        check_hello(PROTOCOL_VERSION, CONTRACT_VERSION, CONTRACT_VERSION),
        Ok(())
    );
}

/// Un client plus récent que le serveur ou d'une autre majeure : contrat refusé.
#[test]
fn a_newer_client_or_other_major_is_refused_as_contract_version() {
    let server = contract(1, 6);
    let refusal = Err(HelloRefusal::ContractVersion);
    assert_eq!(
        check_hello(PROTOCOL_VERSION, contract(1, 7), server),
        refusal
    );
    assert_eq!(
        check_hello(PROTOCOL_VERSION, contract(2, 0), server),
        refusal
    );
    assert_eq!(
        check_hello(PROTOCOL_VERSION, contract(0, 9), server),
        refusal
    );
}

/// Une autre majeure du protocole est refusée, et avant le contrat.
#[test]
fn a_different_protocol_major_is_refused_before_the_contract() {
    let other = ProtocolVersion {
        major: PROTOCOL_VERSION.major + 1,
        minor: 0,
    };
    let refusal = Err(HelloRefusal::ProtocolVersion);
    assert_eq!(check_hello(other, contract(1, 6), contract(1, 6)), refusal);
    assert_eq!(check_hello(other, contract(9, 9), contract(1, 6)), refusal);
}

/// Une mineure plus récente du protocole reste acceptée.
#[test]
fn a_newer_protocol_minor_is_accepted() {
    let newer = ProtocolVersion {
        major: PROTOCOL_VERSION.major,
        minor: PROTOCOL_VERSION.minor + 3,
    };
    assert_eq!(check_hello(newer, contract(1, 6), contract(1, 6)), Ok(()));
}
