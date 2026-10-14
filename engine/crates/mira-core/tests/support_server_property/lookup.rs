//! Retrouver et déposer les messages du faux serveur : un message (clé) y est reconnaissable par son identifiant stable.

use mira_testkit::imap::{FakeMailStore, FakeMessageView};

use crate::support_sync::mail::mail;

pub const USER: &str = "alice";
pub const MAILBOXES: [&str; 3] = ["INBOX", "Archive", "Work"];

/// Identifiant stable du message de clé donnée
pub fn stable_id_of(key: u8) -> u64 {
    100 + u64::from(key)
}

/// Objet du message de clé donnée
pub fn subject_of(key: u8) -> String {
    format!("Message {key}")
}

/// Boîtes (et copies) où le message de clé donnée se trouve
pub fn locations(store: &FakeMailStore, key: u8) -> Vec<(&'static str, FakeMessageView)> {
    let mut found = Vec::new();
    for name in MAILBOXES {
        for view in store.messages(USER, name) {
            if view.gm_msgid == stable_id_of(key) {
                found.push((name, view));
            }
        }
    }
    found
}

/// Dépose le message dans la boîte avec les drapeaux donnés (ils sont portés par le message)
pub fn deliver_copy(store: &FakeMailStore, key: u8, mailbox: &str, flags: &[String]) {
    let raw = mail(&subject_of(key), &format!("k{key}@x"));
    let thread = 500 + u64::from(key);
    let uids = store.deliver_with_labels(USER, &[mailbox], &raw, stable_id_of(key), thread);
    let names: Vec<&str> = flags.iter().map(String::as_str).collect();
    store.set_flags(USER, mailbox, uids[0], &names);
}
