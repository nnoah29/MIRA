#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Propriétés des invariants 10 et 11 : un compte garde ses messages et au moins une identité.

mod support;
mod support_mirror;
mod support_property;

use mira_core::storage::Database;
use mira_vocabulary::AccountId;
use proptest::prelude::*;
use support::accounts::{database_with_account, sample_identity};
use support_property::operations::store_ops;
use support_property::quick_config;
use support_property::store_world::{StoreWorld, account_id};

/// Applique une opération d'identité (ajout, choix du défaut, mise à jour) sur le compte
fn apply_identity_operation(db: &Database, account: &str, kind: u8, pick: u8, serial: usize) {
    let existing = db
        .read(|tx| tx.list_identities(&AccountId::from_raw(account)))
        .unwrap();
    let chosen = existing[usize::from(pick) % existing.len()].clone();
    db.write(|tx| match kind % 3 {
        0 => tx.insert_identity(&sample_identity(
            &format!("{account}-extra-{serial}"),
            account,
            pick % 2 == 0,
        )),
        1 => tx.set_default_identity(&chosen.account, &chosen.id),
        _ => tx.update_identity(&chosen),
    })
    .unwrap();
}

/// Lit les identités du compte
fn defaults_and_total(db: &Database, account: &str) -> (usize, usize) {
    let identities = db
        .read(|tx| tx.list_identities(&AccountId::from_raw(account)))
        .unwrap();
    let defaults = identities.iter().filter(|found| found.is_default).count();
    (identities.len(), defaults)
}

proptest! {
    #![proptest_config(quick_config())]

    /// Invariant 11 : après toute suite d'opérations sur les identités, le compte en a au moins une, dont une seule par défaut
    #[test]
    fn invariant_11_an_account_always_has_one_default_identity(
        steps in proptest::collection::vec((0..3u8, any::<u8>()), 1..30),
    ) {
        let db = database_with_account("acc-1");
        db.write(|tx| tx.insert_identity(&sample_identity("main", "acc-1", true))).unwrap();
        for (serial, (kind, pick)) in steps.into_iter().enumerate() {
            apply_identity_operation(&db, "acc-1", kind, pick, serial);
            let (total, defaults) = defaults_and_total(&db, "acc-1");
            prop_assert!(total >= 1);
            prop_assert_eq!(defaults, 1);
        }
    }

    /// Invariant 10 : supprimer un compte emporte ses messages et laisse intacts ceux de l'autre compte
    #[test]
    fn invariant_10_deleting_an_account_leaves_the_other_account_whole(operations in store_ops()) {
        let mut world = StoreWorld::new();
        for operation in &operations {
            world.apply(operation);
        }
        world.db.write(|tx| tx.delete_account(&account_id(0))).unwrap();
        world.model.messages.retain(|message| message.account != 0);
        support_property::checks::identity::check_internal_ids(&world);
        support_property::checks::identity::check_one_row_per_message(&world);
    }
}
