//! Livraison de messages dans les boîtes.

use std::sync::Arc;

use super::data::{MessageBody, StoreData};

impl StoreData {
    /// Nouveau message avec identifiants Gmail attribués automatiquement
    pub(crate) fn new_body(&mut self, raw: &[u8]) -> Arc<MessageBody> {
        self.message_counter += 1;
        self.body_with_ids(raw, self.message_counter, self.message_counter)
    }

    /// Nouveau message avec identifiants Gmail imposés
    pub(crate) fn body_with_ids(&mut self, raw: &[u8], msgid: u64, thrid: u64) -> Arc<MessageBody> {
        Arc::new(MessageBody {
            raw: raw.to_vec(),
            gm_msgid: msgid,
            gm_thrid: thrid,
        })
    }

    /// Dépose un message existant dans une boîte et renvoie son UID
    pub(crate) fn deliver_body(
        &mut self,
        user: &str,
        mailbox: &str,
        body: &Arc<MessageBody>,
        flags: &[String],
        date: i64,
    ) -> u32 {
        self.mailbox_or_create(user, mailbox)
            .append(flags.to_vec(), date, Arc::clone(body))
    }
}
