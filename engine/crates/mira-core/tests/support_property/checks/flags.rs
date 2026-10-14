//! Invariant 12 : les drapeaux appartiennent au message ; toute boîte les montre à l'identique.

use mira_core::storage::Tx;
use mira_vocabulary::{MiraError, PageRequest};

use crate::support_property::model::BOX_COUNT;
use crate::support_property::store_world::{StoreWorld, mailbox_id};

/// Les drapeaux relus du message, et vus depuis chaque boîte qui le liste, sont ceux du modèle
pub fn check_flags_live_on_message(world: &StoreWorld) {
    world
        .db
        .read(|tx| {
            for message in &world.model.messages {
                assert_eq!(tx.get_message(&message.id)?.unwrap().flags, message.flags);
            }
            for mailbox in 0..BOX_COUNT {
                check_listed_flags(world, tx, mailbox)?;
            }
            Ok(())
        })
        .unwrap();
}

/// La liste d'une boîte montre exactement les apparitions non supprimées, avec les drapeaux du message
fn check_listed_flags(world: &StoreWorld, tx: &Tx<'_>, mailbox: usize) -> Result<(), MiraError> {
    let request = PageRequest {
        cursor: None,
        limit: 500,
    };
    let page = tx.list_messages_in_mailbox(&mailbox_id(mailbox), &request)?;
    let visible = world
        .model
        .members(mailbox)
        .into_iter()
        .filter(|&(_, index)| !world.model.messages[index].appearances[&mailbox].deleted)
        .count();
    assert_eq!(page.items.len(), visible, "liste de la boîte {mailbox}");
    for listed in &page.items {
        let expected = world.model.messages.iter().find(|m| m.id == listed.id);
        assert_eq!(Some(listed.flags), expected.map(|m| m.flags));
    }
    Ok(())
}
