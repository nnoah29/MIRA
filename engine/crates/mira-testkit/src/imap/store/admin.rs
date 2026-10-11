//! Création, suppression et renommage de boîtes.

use super::data::{StoreData, UserData};
use super::names::normalize_name;

impl StoreData {
    /// Crée une boîte ; faux si elle existe déjà
    pub(crate) fn create_named(&mut self, user: &str, name: &str) -> bool {
        if self.mailbox(user, name).is_some() {
            return false;
        }
        self.mailbox_or_create(user, name);
        true
    }

    /// Supprime une boîte (sauf INBOX) ; faux si elle n'existe pas
    pub(crate) fn delete_named(&mut self, user: &str, name: &str) -> bool {
        let key = normalize_name(name);
        if key == "INBOX" {
            return false;
        }
        self.users
            .get_mut(user)
            .is_some_and(|owner| owner.mailboxes.remove(&key).is_some())
    }

    /// Renomme une boîte et ses sous-boîtes
    pub(crate) fn rename_named(
        &mut self,
        user: &str,
        from: &str,
        to: &str,
    ) -> Result<(), &'static str> {
        let (from, to) = (normalize_name(from), normalize_name(to));
        if from == "INBOX" {
            return Err("Cannot rename INBOX");
        }
        if self.mailbox(user, &to).is_some() {
            return Err("[ALREADYEXISTS] Mailbox already exists");
        }
        let owner = self
            .users
            .get_mut(user)
            .ok_or("[NONEXISTENT] No such mailbox")?;
        owner.rename_tree(&from, &to)
    }
}

impl UserData {
    /// Déplace la boîte et ses descendantes sous le nouveau nom
    fn rename_tree(&mut self, from: &str, to: &str) -> Result<(), &'static str> {
        let prefix = format!("{from}/");
        let moved: Vec<String> = self
            .mailboxes
            .keys()
            .filter(|key| *key == from || key.starts_with(&prefix))
            .cloned()
            .collect();
        if moved.is_empty() {
            return Err("[NONEXISTENT] No such mailbox");
        }
        for key in moved {
            if let Some(mailbox) = self.mailboxes.remove(&key) {
                self.mailboxes
                    .insert(format!("{to}{}", &key[from.len()..]), mailbox);
            }
        }
        Ok(())
    }
}
