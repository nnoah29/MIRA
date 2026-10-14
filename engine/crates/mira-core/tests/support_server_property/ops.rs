//! Changements du serveur tirés au hasard : livraison, étiquette, suppression, déplacement, renumérotation, drapeaux.

use proptest::prelude::*;

/// Une action sur le faux serveur ou une synchronisation ; `key` désigne un message (identifiant stable dérivé)
#[derive(Debug, Clone)]
pub enum ServerOp {
    Deliver { key: u8, mask: u8 },
    Label { key: u8, mailbox: usize },
    Expunge { key: u8, mailbox: usize },
    Move { key: u8, from: usize, to: usize },
    Renumber { mailbox: usize },
    SetFlags { key: u8, bits: u8 },
    Sync,
}

/// Changements possibles sur un serveur à libellés (messages sous plusieurs boîtes)
fn labelled_op() -> impl Strategy<Value = ServerOp> {
    prop_oneof![
        4 => (0..5u8, 1..8u8).prop_map(|(key, mask)| ServerOp::Deliver { key, mask }),
        2 => (0..5u8, 0..3usize).prop_map(|(key, mailbox)| ServerOp::Label { key, mailbox }),
        2 => (0..5u8, 0..3usize).prop_map(|(key, mailbox)| ServerOp::Expunge { key, mailbox }),
        3 => (0..5u8, 0..3usize, 0..3usize)
            .prop_map(|(key, from, to)| ServerOp::Move { key, from, to }),
        2 => (0..3usize).prop_map(|mailbox| ServerOp::Renumber { mailbox }),
        2 => (0..5u8, 0..8u8).prop_map(|(key, bits)| ServerOp::SetFlags { key, bits }),
        4 => Just(ServerOp::Sync),
    ]
}

/// Changements possibles sur un serveur à dossiers : tout reste dans la réception, sans déplacement
fn folder_op() -> impl Strategy<Value = ServerOp> {
    prop_oneof![
        4 => (0..5u8).prop_map(|key| ServerOp::Deliver { key, mask: 1 }),
        2 => (0..5u8).prop_map(|key| ServerOp::Expunge { key, mailbox: 0 }),
        2 => Just(ServerOp::Renumber { mailbox: 0 }),
        2 => (0..5u8, 0..8u8).prop_map(|(key, bits)| ServerOp::SetFlags { key, bits }),
        4 => Just(ServerOp::Sync),
    ]
}

/// Suite de changements d'un serveur à libellés
pub fn labelled_server_ops() -> impl Strategy<Value = Vec<ServerOp>> {
    proptest::collection::vec(labelled_op(), 1..24)
}

/// Suite de changements d'un serveur à dossiers
pub fn folder_server_ops() -> impl Strategy<Value = Vec<ServerOp>> {
    proptest::collection::vec(folder_op(), 1..24)
}
