#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Un client plus récent que le moteur : c'est le moteur qui est périmé, pas le client.

use mira_protocol::client_is_newer;
use mira_vocabulary::ContractVersion;

/// Version du contrat avec ces deux nombres.
fn contract(major: u32, minor: u32) -> ContractVersion {
    ContractVersion { major, minor }
}

/// Même majeure et mineure supérieure, ou majeure supérieure : le client est plus récent.
#[test]
fn a_higher_minor_or_major_is_newer() {
    let server = contract(1, 6);
    assert!(client_is_newer(server, contract(1, 7)));
    assert!(client_is_newer(server, contract(2, 0)));
}

/// Même version, mineure inférieure ou majeure inférieure : le client n'est pas plus récent.
#[test]
fn equal_or_lower_versions_are_not_newer() {
    let server = contract(1, 6);
    assert!(!client_is_newer(server, contract(1, 6)));
    assert!(!client_is_newer(server, contract(1, 0)));
    assert!(!client_is_newer(server, contract(0, 9)));
}
