//! Simulations de ce que fait une synchro (le serveur gagne) et du rebase qui suit.

use mira_core::actions::reapply_pending_local_effects;
use mira_vocabulary::{AccountId, Flags, MailboxId, MessageId};

use super::world::World;
use crate::support_mirror::fixtures::appearance;

/// Rejoue les effets des actions en attente du compte acc-1
pub fn rebase(world: &World) {
    let account = AccountId::from_raw("acc-1");
    world
        .db
        .write(|tx| reapply_pending_local_effects(tx, &account))
        .unwrap();
}

/// Simule une synchro qui écrase les drapeaux du message avec l'état du serveur
pub fn server_overwrites_flags(world: &World, id: &str, flags: Flags) {
    world
        .db
        .write(|tx| tx.update_message_flags(&MessageId::from_raw(id), flags))
        .unwrap();
}

/// Simule une synchro : msg-1 n'est plus dans la réception ni chez « Projects », il est dans « Work » (UID 9)
pub fn server_moves_msg_1_to_work(world: &World) {
    let message = MessageId::from_raw("msg-1");
    world
        .db
        .write(|tx| {
            tx.remove_appearance(&message, &MailboxId::from_raw("box-in"))?;
            tx.remove_appearance(&message, &MailboxId::from_raw("box-proj"))?;
            tx.add_appearance(&appearance("msg-1", "box-work", 9))?;
            tx.refresh_mailbox_counters(&MailboxId::from_raw("box-work"))?;
            tx.refresh_mailbox_counters(&MailboxId::from_raw("box-proj"))
        })
        .unwrap();
}
