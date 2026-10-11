#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Capacités effectives : le serveur est la vérité, le provider ne retire que des exceptions (D-08).

use mira_core::capabilities::{EffectiveCapabilities, ServerCapability, parse_capabilities};
use mira_vocabulary::ProviderQuirks;

/// Calcule les capacités effectives pour des jetons annoncés et des extensions peu fiables
fn effective(announced: &[&str], unreliable: &[&str]) -> EffectiveCapabilities {
    let tokens: Vec<String> = announced.iter().map(ToString::to_string).collect();
    let quirks = ProviderQuirks {
        unreliable_extensions: unreliable.iter().map(ToString::to_string).collect(),
        ..ProviderQuirks::default()
    };
    EffectiveCapabilities::compute(&parse_capabilities(&tokens), &quirks)
}

/// Sans exception déclarée, les capacités effectives sont celles du serveur
#[test]
fn without_exceptions_the_server_announcement_stands() {
    let capabilities = effective(&["IDLE", "MOVE", "QRESYNC"], &[]);
    assert!(capabilities.has(ServerCapability::Idle));
    assert!(capabilities.has(ServerCapability::Move));
    assert!(capabilities.has(ServerCapability::Qresync));
    assert!(!capabilities.has(ServerCapability::UidPlus));
    assert_eq!(capabilities.all().len(), 3);
}

/// Une extension déclarée peu fiable est retirée de l'annonce, casse ignorée
#[test]
fn provider_exception_removes_an_announced_extension() {
    let capabilities = effective(
        &["IDLE", "MOVE", "QRESYNC", "CONDSTORE"],
        &["qresync", "Move"],
    );
    assert!(!capabilities.has(ServerCapability::Qresync));
    assert!(!capabilities.has(ServerCapability::Move));
    assert!(capabilities.has(ServerCapability::Idle));
    assert!(capabilities.has(ServerCapability::Condstore));
}

/// Le provider ne peut pas ajouter une extension que le serveur n'annonce pas
#[test]
fn provider_cannot_add_what_the_server_does_not_announce() {
    let capabilities = effective(&["IDLE"], &["MOVE", "NOM-INCONNU"]);
    assert!(capabilities.has(ServerCapability::Idle));
    assert!(!capabilities.has(ServerCapability::Move));
    assert_eq!(capabilities.all().len(), 1);
}

/// COMPRESS=DEFLATE peut être retiré par son nom complet ou abrégé
#[test]
fn compress_can_be_removed_by_either_name() {
    assert!(
        !effective(&["COMPRESS=DEFLATE"], &["COMPRESS=DEFLATE"]).has(ServerCapability::Compress)
    );
    assert!(!effective(&["COMPRESS=DEFLATE"], &["COMPRESS"]).has(ServerCapability::Compress));
}
