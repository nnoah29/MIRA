//! Structures de données du magasin.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

/// Contenu partagé d'un message : source brute et identifiants Gmail.
#[derive(Debug)]
pub(crate) struct MessageBody {
    pub raw: Vec<u8>,
    pub gm_msgid: u64,
    pub gm_thrid: u64,
}

/// Présence d'un message dans une boîte.
#[derive(Debug, Clone)]
pub(crate) struct Entry {
    pub uid: u32,
    pub modseq: u64,
    pub flags: Vec<String>,
    pub internal_date: i64,
    pub message: Arc<MessageBody>,
}

/// Trace d'un message supprimé, pour VANISHED.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Expunged {
    pub uid: u32,
    pub modseq: u64,
}

/// Boîte aux lettres d'un utilisateur.
#[derive(Debug)]
pub(crate) struct Mailbox {
    pub special_use: Option<String>,
    pub uidvalidity: u32,
    pub uidnext: u32,
    pub highest_modseq: u64,
    pub entries: Vec<Entry>,
    pub expunged: Vec<Expunged>,
}

/// Boîtes d'un utilisateur, indexées par nom.
#[derive(Debug, Default)]
pub(crate) struct UserData {
    pub mailboxes: BTreeMap<String, Mailbox>,
}

/// Contenu complet du magasin.
#[derive(Debug, Default)]
pub(crate) struct StoreData {
    pub users: HashMap<String, UserData>,
    pub validity_counter: u32,
    pub message_counter: u64,
}
