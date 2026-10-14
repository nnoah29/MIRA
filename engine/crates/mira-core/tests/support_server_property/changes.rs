//! Changements appliqués au faux serveur par les séquences aléatoires.

use mira_testkit::imap::FakeMailStore;

use super::lookup::{MAILBOXES, USER, deliver_copy, locations};
use super::ops::ServerOp;

/// Étiquette le message dans une boîte où il n'est pas encore
fn label(store: &FakeMailStore, key: u8, mailbox: usize) {
    let found = locations(store, key);
    let name = MAILBOXES[mailbox];
    if let Some((_, first)) = found.first()
        && found.iter().all(|(other, _)| *other != name)
    {
        deliver_copy(store, key, name, &first.flags);
    }
}

/// Retire la copie du message dans une boîte
fn expunge(store: &FakeMailStore, key: u8, mailbox: usize) {
    for (name, view) in locations(store, key) {
        if name == MAILBOXES[mailbox] {
            store.expunge(USER, name, view.uid);
        }
    }
}

/// Livre un nouveau message dans les boîtes du masque ; rien si la clé existe déjà
fn deliver(store: &FakeMailStore, key: u8, mask: u8) {
    if !locations(store, key).is_empty() {
        return;
    }
    for (bit, name) in MAILBOXES.iter().enumerate() {
        if mask >> bit & 1 == 1 {
            deliver_copy(store, key, name, &[]);
        }
    }
}

/// Remplace les drapeaux du message dans toutes ses boîtes
fn set_flags(store: &FakeMailStore, key: u8, bits: u8) {
    let all = ["\\Seen", "\\Flagged", "\\Answered"];
    let chosen: Vec<&str> = (0..3)
        .filter(|bit| bits >> bit & 1 == 1)
        .map(|bit| all[bit])
        .collect();
    for (name, view) in locations(store, key) {
        store.set_flags(USER, name, view.uid, &chosen);
    }
}

/// Applique le changement au serveur ; la synchronisation est faite par l'appelant
pub fn perform(store: &FakeMailStore, operation: &ServerOp) {
    match *operation {
        ServerOp::Deliver { key, mask } => deliver(store, key, mask),
        ServerOp::Label { key, mailbox } => label(store, key, mailbox),
        ServerOp::Expunge { key, mailbox } => expunge(store, key, mailbox),
        ServerOp::Move { key, from, to } if from != to => {
            label(store, key, to);
            expunge(store, key, from);
        }
        ServerOp::Move { .. } | ServerOp::Sync => {}
        ServerOp::Renumber { mailbox } => store.renumber(USER, MAILBOXES[mailbox]),
        ServerOp::SetFlags { key, bits } => set_flags(store, key, bits),
    }
}
