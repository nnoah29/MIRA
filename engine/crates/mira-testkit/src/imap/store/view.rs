//! Vue instantanée d'une boîte pour SELECT et la synchronisation de session.

use super::data::StoreData;
use super::rows::MessageRow;

/// État d'une boîte et changements depuis un MODSEQ donné.
#[derive(Debug, Clone)]
pub(crate) struct MailboxView {
    pub uids: Vec<u32>,
    pub validity: u32,
    pub uidnext: u32,
    pub highest_modseq: u64,
    pub flags: Vec<String>,
    pub unseen: u32,
    pub changed: Vec<MessageRow>,
    pub vanished: Vec<u32>,
}

impl StoreData {
    /// Vue de la boîte avec les changements dont le MODSEQ dépasse `since`
    pub(crate) fn view(&self, user: &str, name: &str, since: u64) -> Option<MailboxView> {
        let mailbox = self.mailbox(user, name)?;
        let changed: Vec<u32> = mailbox
            .entries
            .iter()
            .filter(|entry| entry.modseq > since)
            .map(|entry| entry.uid)
            .collect();
        let mut flags: Vec<String> = mailbox
            .entries
            .iter()
            .flat_map(|entry| entry.flags.iter().cloned())
            .collect();
        flags.sort();
        flags.dedup();
        Some(MailboxView {
            uids: mailbox.uids(),
            validity: mailbox.uidvalidity,
            uidnext: mailbox.uidnext,
            highest_modseq: mailbox.highest_modseq,
            unseen: count_unseen(mailbox),
            flags,
            changed: self.rows(user, name, &changed, false),
            vanished: vanished_since(mailbox, since),
        })
    }
}

/// Nombre de messages sans le drapeau `\Seen`
fn count_unseen(mailbox: &super::data::Mailbox) -> u32 {
    let unseen =
        |flags: &Vec<String>| !flags.iter().any(|flag| flag.eq_ignore_ascii_case("\\Seen"));
    mailbox
        .entries
        .iter()
        .filter(|entry| unseen(&entry.flags))
        .count() as u32
}

/// UID supprimés après le MODSEQ indiqué
fn vanished_since(mailbox: &super::data::Mailbox, since: u64) -> Vec<u32> {
    mailbox
        .expunged
        .iter()
        .filter(|gone| gone.modseq > since)
        .map(|gone| gone.uid)
        .collect()
}
