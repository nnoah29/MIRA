#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Les outils d'un fournisseur local dépendent du modèle choisi ; les autres suivent leur fiche.

use mira_core::ai::local::tools_allowed;

mod local_support;
use local_support::{descriptor, test_runtime};

/// Le palier Avancé a les outils, le palier Mini non, un modèle inconnu non
#[test]
fn tools_follow_the_model_for_a_local_provider() {
    let local = descriptor("local", false, Some(test_runtime()));
    assert!(tools_allowed(&local, "big"));
    assert!(!tools_allowed(&local, "mini"));
    assert!(!tools_allowed(&local, "inconnu"));
}

/// Un fournisseur sans modèle local garde le réglage de sa fiche, quel que soit le modèle
#[test]
fn non_local_providers_keep_the_descriptor_flag() {
    assert!(tools_allowed(
        &descriptor("plain", true, None),
        "n-importe-lequel"
    ));
    assert!(!tools_allowed(
        &descriptor("plain", false, None),
        "n-importe-lequel"
    ));
}
