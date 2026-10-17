//! Modèle de référence du miroir : ce que la base doit contenir après chaque opération.

use std::collections::BTreeMap;

use mira_vocabulary::{Flags, MessageId};

pub const ACCOUNTS: usize = 2;
pub const BOXES_PER_ACCOUNT: usize = 3;
pub const BOX_COUNT: usize = ACCOUNTS * BOXES_PER_ACCOUNT;

/// Apparition attendue d'un message dans une boîte
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelAppearance {
    pub uid: u32,
    pub deleted: bool,
}

/// Message attendu : identifiant interne attribué une fois pour toutes
#[derive(Debug, Clone)]
pub struct ModelMessage {
    pub id: MessageId,
    pub account: usize,
    pub stable: String,
    pub flags: Flags,
    pub appearances: BTreeMap<usize, ModelAppearance>,
}

/// État attendu du miroir : messages vivants et prochain UID de chaque boîte
#[derive(Debug, Clone)]
pub struct Model {
    pub messages: Vec<ModelMessage>,
    pub next_uid: [u32; BOX_COUNT],
    pub created: u32,
}

impl Model {
    /// Modèle vide : aucun message, UID à partir de 1
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            next_uid: [1; BOX_COUNT],
            created: 0,
        }
    }

    /// Prend un UID neuf dans la boîte (numérotation globale)
    pub fn take_uid(&mut self, mailbox: usize) -> u32 {
        let uid = self.next_uid[mailbox];
        self.next_uid[mailbox] += 1;
        uid
    }

    /// Apparitions d'une boîte sous la forme (UID, position du message), par UID croissant
    pub fn members(&self, mailbox: usize) -> Vec<(u32, usize)> {
        let mut members = Vec::new();
        for (index, message) in self.messages.iter().enumerate() {
            if let Some(found) = message.appearances.get(&mailbox) {
                members.push((found.uid, index));
            }
        }
        members.sort();
        members
    }

    /// Position du message désigné par un indice quelconque, s'il y a des messages
    pub fn pick(&self, index: usize) -> Option<usize> {
        match self.messages.len() {
            0 => None,
            count => Some(index % count),
        }
    }
}

/// Compte (numéro) auquel appartient une boîte de numérotation globale
pub fn account_of_box(mailbox: usize) -> usize {
    mailbox / BOXES_PER_ACCOUNT
}
