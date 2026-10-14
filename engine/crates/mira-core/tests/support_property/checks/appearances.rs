//! Invariants 8 et 9 : au moins une apparition, boîte du même compte ; les boîtes sont des vues sur les apparitions.

use std::collections::BTreeMap;

use crate::support_property::model::{BOX_COUNT, account_of_box};
use crate::support_property::store_world::{StoreWorld, account_id, mailbox_id};

/// Apparitions stockées de chaque boîte comparées au modèle : même message, même UID, même état, UID distincts
pub fn check_appearances_match_model(world: &StoreWorld) {
    world
        .db
        .read(|tx| {
            for mailbox in 0..BOX_COUNT {
                let stored = tx.appearances_in_mailbox(&mailbox_id(mailbox))?;
                let mut seen: BTreeMap<u32, String> = BTreeMap::new();
                for found in &stored {
                    assert!(!found.needs_revalidation && found.uid > 0);
                    let clash = seen.insert(found.uid, found.message.to_string());
                    assert!(clash.is_none(), "UID en double dans la boîte {mailbox}");
                }
                let expected = world.model.members(mailbox);
                assert_eq!(stored.len(), expected.len(), "boîte {mailbox}");
                for (uid, index) in expected {
                    let message = &world.model.messages[index];
                    let found = tx
                        .get_appearance(&message.id, &mailbox_id(mailbox))?
                        .unwrap();
                    assert_eq!(
                        (found.uid, found.deleted),
                        (uid, message.appearances[&mailbox].deleted)
                    );
                }
            }
            Ok(())
        })
        .unwrap();
}

/// Invariant 9 : chaque apparition référence une boîte du compte du message
pub fn check_appearance_accounts(world: &StoreWorld) {
    world
        .db
        .read(|tx| {
            for mailbox in 0..BOX_COUNT {
                let owner = tx.get_mailbox(&mailbox_id(mailbox))?.unwrap().account;
                assert_eq!(owner, account_id(account_of_box(mailbox)));
                for found in tx.appearances_in_mailbox(&mailbox_id(mailbox))? {
                    let message = tx.get_message(&found.message)?.unwrap();
                    assert_eq!(message.account, owner, "apparition d'un autre compte");
                }
            }
            Ok(())
        })
        .unwrap();
}

/// Invariant 8 : tout message du miroir a au moins une apparition (à appeler après la fin de passe)
pub fn check_every_message_appears(world: &StoreWorld) {
    world
        .db
        .read(|tx| {
            for message in &world.model.messages {
                assert!(!message.appearances.is_empty());
                assert!(!tx.appearances_of_message(&message.id)?.is_empty());
            }
            Ok(())
        })
        .unwrap();
}

/// Les boîtes visibles d'un message sont exactement celles où il apparaît sans être marqué supprimé
pub fn check_mailboxes_are_views(world: &StoreWorld) {
    world
        .db
        .read(|tx| {
            for message in &world.model.messages {
                let shown = tx.get_message(&message.id)?.unwrap().mailboxes;
                let mut expected: Vec<_> = message
                    .appearances
                    .iter()
                    .filter(|(_, found)| !found.deleted)
                    .map(|(&mailbox, _)| mailbox_id(mailbox))
                    .collect();
                expected.sort_by(|left, right| left.as_str().cmp(right.as_str()));
                assert_eq!(shown, expected, "boîtes visibles de {}", message.id);
            }
            Ok(())
        })
        .unwrap();
}
