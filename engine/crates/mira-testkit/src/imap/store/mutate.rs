//! API publique de modification du magasin pour les tests.

use super::entry_flags::{FlagMode, owned};
use super::handle::FakeMailStore;
use crate::imap::date::clock::now_epoch;

impl FakeMailStore {
    /// Crée une boîte (avec attribut SPECIAL-USE facultatif) si elle n'existe pas
    pub fn create_mailbox(&self, user: &str, name: &str, special_use: Option<&str>) {
        self.write(|data| {
            let mailbox = data.mailbox_or_create(user, name);
            if special_use.is_some() {
                mailbox.special_use = special_use.map(str::to_string);
            }
        });
    }

    /// Dépose un message dans une boîte et renvoie son UID ; réveille les IDLE
    pub fn deliver(&self, user: &str, mailbox: &str, raw: &[u8], flags: &[&str]) -> u32 {
        self.deliver_dated(user, mailbox, raw, flags, now_epoch())
    }

    /// Comme `deliver`, avec une date interne imposée (secondes Unix)
    pub fn deliver_dated(
        &self,
        user: &str,
        mailbox: &str,
        raw: &[u8],
        flags: &[&str],
        date: i64,
    ) -> u32 {
        self.write(|data| {
            let body = data.new_body(raw);
            data.deliver_body(user, mailbox, &body, &owned(flags), date)
        })
    }

    /// Dépose le même message dans plusieurs boîtes (modèle Gmail) et renvoie les UID
    pub fn deliver_gmail(
        &self,
        user: &str,
        labels_mailboxes: &[&str],
        raw: &[u8],
        gm_msgid: u64,
        gm_thrid: u64,
    ) -> Vec<u32> {
        self.write(|data| {
            let body = data.body_with_ids(raw, gm_msgid, gm_thrid);
            labels_mailboxes
                .iter()
                .map(|name| data.deliver_body(user, name, &body, &[], now_epoch()))
                .collect()
        })
    }

    /// Remplace les drapeaux d'un message (incrémente MODSEQ)
    pub fn set_flags(&self, user: &str, mailbox: &str, uid: u32, flags: &[&str]) {
        self.write(|data| {
            if let Some(found) = data.mailbox_mut(user, mailbox) {
                found.change_flags(uid, FlagMode::Replace, &owned(flags));
            }
        });
    }

    /// Supprime un message en gardant sa trace pour VANISHED
    pub fn expunge(&self, user: &str, mailbox: &str, uid: u32) {
        self.write(|data| {
            if let Some(found) = data.mailbox_mut(user, mailbox) {
                found.remove_uids(&[uid]);
            }
        });
    }
}
