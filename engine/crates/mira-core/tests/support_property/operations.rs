//! Opérations de stockage tirées au hasard : insertions, étiquettes, déplacements, renumérotations, drapeaux, suppressions.

use proptest::prelude::*;

use super::model::{ACCOUNTS, BOX_COUNT, BOXES_PER_ACCOUNT};

/// Une opération du miroir ; les indices de message sont pris modulo le nombre de messages vivants
#[derive(Debug, Clone)]
pub enum StoreOp {
    Ingest {
        account: usize,
        mailbox: usize,
        stable: u8,
    },
    Label {
        message: usize,
        mailbox: usize,
    },
    Move {
        message: usize,
        to: usize,
    },
    Unlabel {
        message: usize,
        mailbox: usize,
    },
    Renumber {
        mailbox: usize,
        keep_mask: u8,
    },
    SetFlags {
        message: usize,
        bits: u8,
    },
    Trash {
        message: usize,
        mailbox: usize,
        deleted: bool,
    },
    EndOfPass {
        account: usize,
    },
    Delete {
        message: usize,
    },
}

/// Opérations qui créent ou rangent des messages
fn placing_op() -> impl Strategy<Value = StoreOp> {
    prop_oneof![
        5 => (0..ACCOUNTS, 0..BOXES_PER_ACCOUNT, 0..6u8)
            .prop_map(|(account, mailbox, stable)| StoreOp::Ingest { account, mailbox, stable }),
        3 => (0..32usize, 0..BOX_COUNT)
            .prop_map(|(message, mailbox)| StoreOp::Label { message, mailbox }),
        3 => (0..32usize, 0..BOXES_PER_ACCOUNT)
            .prop_map(|(message, to)| StoreOp::Move { message, to }),
        2 => (0..32usize, 0..BOX_COUNT)
            .prop_map(|(message, mailbox)| StoreOp::Unlabel { message, mailbox }),
    ]
}

/// Opérations qui changent l'état d'un message ou d'une boîte
fn changing_op() -> impl Strategy<Value = StoreOp> {
    prop_oneof![
        2 => (0..BOX_COUNT, any::<u8>())
            .prop_map(|(mailbox, keep_mask)| StoreOp::Renumber { mailbox, keep_mask }),
        3 => (0..32usize, 0..16u8)
            .prop_map(|(message, bits)| StoreOp::SetFlags { message, bits }),
        2 => (0..32usize, 0..BOX_COUNT, any::<bool>())
            .prop_map(|(message, mailbox, deleted)| StoreOp::Trash { message, mailbox, deleted }),
        2 => (0..ACCOUNTS).prop_map(|account| StoreOp::EndOfPass { account }),
        1 => (0..32usize).prop_map(|message| StoreOp::Delete { message }),
    ]
}

/// Opération quelconque du miroir
pub fn store_op() -> impl Strategy<Value = StoreOp> {
    prop_oneof![placing_op(), changing_op()]
}

/// Suite d'opérations de longueur modérée
pub fn store_ops() -> impl Strategy<Value = Vec<StoreOp>> {
    proptest::collection::vec(store_op(), 1..40)
}

/// Suite d'opérations qui ne font que ranger des messages (sans suppression ni drapeaux)
pub fn placing_ops() -> impl Strategy<Value = Vec<StoreOp>> {
    proptest::collection::vec(placing_op(), 1..40)
}
