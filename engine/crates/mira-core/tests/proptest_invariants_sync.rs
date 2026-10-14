#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Propriétés des invariants 7, 8 et 12 face à un vrai serveur de test : livraisons, déplacements, renumérotations, drapeaux.

mod support_server_property;
mod support_sync;

use proptest::prelude::*;
use support_server_property::ops::{folder_server_ops, labelled_server_ops};
use support_server_property::run_server_sequence;

proptest! {
    #![proptest_config(ProptestConfig { cases: 40, failure_persistence: None, ..ProptestConfig::default() })]

    /// Invariants 7, 8, 12 (serveur à libellés) : le miroir égale le serveur, sans orphelin, et les messages gardent leur identifiant
    #[test]
    fn invariant_7_labelled_server_changes_keep_internal_ids(operations in labelled_server_ops()) {
        run_server_sequence(&operations, true);
    }

    /// Invariants 7, 8, 12 (serveur à dossiers) : suppressions, drapeaux et renumérotations gardent les identifiants internes
    #[test]
    fn invariant_7_folder_server_changes_keep_internal_ids(operations in folder_server_ops()) {
        run_server_sequence(&operations, false);
    }
}
