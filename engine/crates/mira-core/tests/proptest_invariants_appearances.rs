#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Propriétés des invariants 8 et 9 : apparitions, comptes des boîtes, boîtes comme vues, UID uniques.

mod support;
mod support_mirror;
mod support_property;

use proptest::prelude::*;
use support_property::checks::appearances::{
    check_appearance_accounts, check_appearances_match_model, check_every_message_appears,
    check_mailboxes_are_views,
};
use support_property::operations::store_ops;
use support_property::quick_config;
use support_property::store_world::StoreWorld;

proptest! {
    #![proptest_config(quick_config())]

    /// Invariant 8 : une fois les passes terminées, aucun message ne reste sans apparition
    #[test]
    fn invariant_8_no_message_survives_without_an_appearance(operations in store_ops()) {
        let mut world = StoreWorld::new();
        for operation in &operations {
            world.apply(operation);
        }
        world.finish_passes();
        check_every_message_appears(&world);
        check_appearances_match_model(&world);
    }

    /// Invariant 9 : une apparition est toujours dans une boîte du compte du message, et une boîte d'un autre compte est refusée
    #[test]
    fn invariant_9_appearances_stay_in_the_account_of_their_message(operations in store_ops()) {
        let mut world = StoreWorld::new();
        for operation in &operations {
            world.apply(operation);
            check_appearance_accounts(&world);
        }
    }

    /// Les boîtes sont des vues : apparitions, UID uniques et boîtes visibles suivent le modèle à chaque pas
    #[test]
    fn mailboxes_are_views_over_unique_appearances(operations in store_ops()) {
        let mut world = StoreWorld::new();
        for operation in &operations {
            world.apply(operation);
            check_appearances_match_model(&world);
            check_mailboxes_are_views(&world);
        }
    }
}
