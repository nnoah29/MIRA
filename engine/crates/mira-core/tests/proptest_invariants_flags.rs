#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Propriétés de l'invariant 12 : les drapeaux sont portés par le message, le seul drapeau d'apparition est « supprimé ».

mod support;
mod support_mirror;
mod support_property;

use mira_vocabulary::Flags;
use proptest::prelude::*;
use support_property::checks::flags::check_flags_live_on_message;
use support_property::operations::{StoreOp, store_ops};
use support_property::quick_config;
use support_property::store_world::StoreWorld;

/// Drapeaux de chaque message, relus dans la base
fn stored_flags(world: &StoreWorld) -> Vec<Flags> {
    let read = |tx: &mira_core::storage::Tx<'_>| {
        let mut all = Vec::new();
        for message in &world.model.messages {
            all.push(tx.get_message(&message.id)?.unwrap().flags);
        }
        Ok(all)
    };
    world.db.read(read).unwrap()
}

proptest! {
    #![proptest_config(quick_config())]

    /// Invariant 12 : quelles que soient les opérations, le message montre les mêmes drapeaux dans toutes ses boîtes
    #[test]
    fn invariant_12_flags_are_the_same_in_every_mailbox(operations in store_ops()) {
        let mut world = StoreWorld::new();
        for operation in &operations {
            world.apply(operation);
            check_flags_live_on_message(&world);
        }
    }

    /// Invariant 12 : marquer une apparition « supprimée » ne change aucun drapeau du message
    #[test]
    fn invariant_12_deleting_an_appearance_leaves_message_flags_alone(
        operations in store_ops(),
        message in 0..32usize,
        mailbox in 0..6usize,
    ) {
        let mut world = StoreWorld::new();
        for operation in &operations {
            world.apply(operation);
        }
        let before = stored_flags(&world);
        world.apply(&StoreOp::Trash { message, mailbox, deleted: true });
        let after = stored_flags(&world);
        prop_assert_eq!(before, after);
        check_flags_live_on_message(&world);
    }
}
