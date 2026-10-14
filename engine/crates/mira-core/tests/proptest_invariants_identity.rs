#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Propriétés des invariants 7 et 10 : identifiant interne stable, un message = une ligne, un seul compte.

mod support;
mod support_mirror;
mod support_property;

use proptest::prelude::*;
use support_property::checks::identity::{
    check_internal_ids, check_one_row_per_message, check_ownership,
};
use support_property::operations::{placing_ops, store_ops};
use support_property::quick_config;
use support_property::store_world::StoreWorld;

proptest! {
    #![proptest_config(quick_config())]

    /// Invariant 7 : après chaque opération (arrivée, étiquette, déplacement, renumérotation, suppression), chaque message garde son identifiant
    #[test]
    fn invariant_7_internal_ids_survive_any_sequence_of_operations(operations in store_ops()) {
        let mut world = StoreWorld::new();
        for operation in &operations {
            world.apply(operation);
            check_internal_ids(&world);
        }
    }

    /// Un message = une ligne : arrivées répétées, étiquettes et déplacements ne dupliquent jamais un message
    #[test]
    fn one_message_is_one_row_whatever_the_labels_and_moves(operations in placing_ops()) {
        let mut world = StoreWorld::new();
        for operation in &operations {
            world.apply(operation);
            check_one_row_per_message(&world);
        }
    }

    /// Invariant 10 : un message appartient à un seul compte, même quand deux comptes reçoivent le même identifiant stable
    #[test]
    fn invariant_10_a_message_belongs_to_exactly_one_account(operations in store_ops()) {
        let mut world = StoreWorld::new();
        for operation in &operations {
            world.apply(operation);
        }
        check_ownership(&world);
        check_one_row_per_message(&world);
    }
}
