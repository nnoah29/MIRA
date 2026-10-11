//! Accès aux boîtes d'un utilisateur.

use super::data::{Mailbox, StoreData, UserData};
use super::names::normalize_name;

/// Prend le prochain UIDVALIDITY du compteur
pub(crate) fn take_validity(counter: &mut u32) -> u32 {
    *counter += 1;
    1000 + *counter
}

impl UserData {
    /// Utilisateur neuf, qui possède déjà sa boîte INBOX
    pub(crate) fn with_inbox(validity: u32) -> Self {
        let mut user = Self::default();
        user.mailboxes
            .insert("INBOX".to_string(), Mailbox::new(validity, None));
        user
    }
}

impl StoreData {
    /// Boîte existante d'un utilisateur
    pub(crate) fn mailbox(&self, user: &str, name: &str) -> Option<&Mailbox> {
        self.users.get(user)?.mailboxes.get(&normalize_name(name))
    }

    /// Boîte existante d'un utilisateur, modifiable
    pub(crate) fn mailbox_mut(&mut self, user: &str, name: &str) -> Option<&mut Mailbox> {
        self.users
            .get_mut(user)?
            .mailboxes
            .get_mut(&normalize_name(name))
    }

    /// Boîte d'un utilisateur, créée avec l'utilisateur si nécessaire
    pub(crate) fn mailbox_or_create(&mut self, user: &str, name: &str) -> &mut Mailbox {
        let Self {
            users,
            validity_counter,
            ..
        } = self;
        let user_data = users
            .entry(user.to_string())
            .or_insert_with(|| UserData::with_inbox(take_validity(validity_counter)));
        user_data
            .mailboxes
            .entry(normalize_name(name))
            .or_insert_with(|| Mailbox::new(take_validity(validity_counter), None))
    }

    /// Données d'un utilisateur, créées avec INBOX si nécessaire
    pub(crate) fn user_mut(&mut self, user: &str) -> &mut UserData {
        self.mailbox_or_create(user, "INBOX");
        self.users.entry(user.to_string()).or_default()
    }
}
