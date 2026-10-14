//! Arrivée d'un message du serveur (reconnu par identifiant stable ou créé), étiquettes ajoutées et retirées.

use std::collections::BTreeMap;

use mira_core::storage::repositories::server_ids::ServerIdKind;
use mira_vocabulary::{Flags, MessageId, Timestamp};

use super::{StoreWorld, account_id, mailbox_id};
use crate::support_mirror::fixtures::{appearance, sample_message};
use crate::support_property::model::{ModelAppearance, ModelMessage, account_of_box};

impl StoreWorld {
    /// Reconnaît le message par son identifiant stable ou le crée, puis l'ajoute à la boîte
    pub(super) fn apply_ingest(&mut self, account: usize, mailbox: usize, stable: u8) {
        let global = account * super::BOXES_PER_ACCOUNT + mailbox;
        let value = format!("gm-{stable}");
        let known = self
            .db
            .read(|tx| tx.find_by_server_id(&account_id(account), ServerIdKind::Stable, &value))
            .unwrap();
        let position = self
            .model
            .messages
            .iter()
            .position(|message| message.account == account && message.stable == value);
        assert_eq!(
            known.is_some(),
            position.is_some(),
            "table des identifiants"
        );
        match position {
            Some(index) => self.place(index, global),
            None => self.create(account, global, &value),
        }
    }

    /// Crée le message, son apparition et sa correspondance serveur dans une seule transaction
    fn create(&mut self, account: usize, global: usize, stable: &str) {
        self.model.created += 1;
        let id = MessageId::from_raw(format!("msg-{}", self.model.created));
        let uid = self.model.take_uid(global);
        self.db
            .write(|tx| {
                tx.insert_message(
                    &sample_message(id.as_str(), account_id(account).as_str()),
                    Timestamp::from_millis(i64::from(self.model.created) * 1_000),
                )?;
                tx.add_appearance(&appearance(id.as_str(), mailbox_id(global).as_str(), uid))?;
                tx.map_server_id(&account_id(account), ServerIdKind::Stable, stable, &id)
            })
            .unwrap();
        let appearances = BTreeMap::from([(
            global,
            ModelAppearance {
                uid,
                deleted: false,
            },
        )]);
        self.model.messages.push(ModelMessage {
            id,
            account,
            stable: stable.to_string(),
            flags: Flags::default(),
            appearances,
        });
    }

    /// Ajoute l'apparition d'un message connu dans une boîte de son compte, si elle manque
    pub(super) fn place(&mut self, index: usize, global: usize) {
        if self.model.messages[index].appearances.contains_key(&global) {
            return;
        }
        let uid = self.model.take_uid(global);
        let id = self.model.messages[index].id.clone();
        self.db
            .write(|tx| {
                tx.add_appearance(&appearance(id.as_str(), mailbox_id(global).as_str(), uid))
            })
            .unwrap();
        let entry = ModelAppearance {
            uid,
            deleted: false,
        };
        self.model.messages[index].appearances.insert(global, entry);
    }

    /// Ajoute une étiquette ; une boîte d'un autre compte est refusée et ne change rien
    pub(super) fn apply_label(&mut self, message: usize, mailbox: usize) {
        let Some(index) = self.model.pick(message) else {
            return;
        };
        if self.model.messages[index].account == account_of_box(mailbox) {
            return self.place(index, mailbox);
        }
        let id = self.model.messages[index].id.clone();
        let outcome = self.db.write(|tx| {
            tx.add_appearance(&appearance(
                id.as_str(),
                mailbox_id(mailbox).as_str(),
                9_999,
            ))
        });
        let error = outcome.expect_err("une boîte d'un autre compte doit être refusée");
        assert_eq!(error.category, mira_vocabulary::ErrorCategory::InvalidInput);
    }

    /// Retire l'apparition d'un message dans une boîte, sans purger le message
    pub(super) fn apply_unlabel(&mut self, message: usize, mailbox: usize) {
        let Some(index) = self.model.pick(message) else {
            return;
        };
        let id = self.model.messages[index].id.clone();
        self.db
            .write(|tx| tx.remove_appearance(&id, &mailbox_id(mailbox)))
            .unwrap();
        self.model.messages[index].appearances.remove(&mailbox);
    }
}
