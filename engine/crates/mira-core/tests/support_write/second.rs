//! Second compte du profil (`other.test`) avec une réception et un message, pour les lots multi-comptes.

use mira_vocabulary::{AccountId, AuthMethodKind, MailboxId, MailboxRole, SyncMarker, Timestamp};

use super::fixture::Fixture;
use super::{PASSWORD, SECOND_USER};
use crate::support_api::requests::{add_account, draft};
use crate::support_imap::mail::mail;
use crate::support_mirror::fixtures::{appearance, sample_mailbox, sample_message};

impl Fixture {
    /// Ajoute le second compte, son courrier sur le serveur (msg-b1, UID 1) et son miroir
    pub async fn add_second_account(&self) -> AccountId {
        self.credentials.add_password(SECOND_USER, PASSWORD);
        let raw = mail("Sujet msg-b1", "msg-b1@example.org");
        self.store.deliver(SECOND_USER, "INBOX", &raw, &[]);
        let request = draft(SECOND_USER, AuthMethodKind::Password);
        let account = add_account(self.engine(), request, PASSWORD, "k-second").await;
        let owner = account.id.as_str();
        let validity = self.store.uidvalidity(SECOND_USER, "INBOX").unwrap();
        self.db
            .write(|tx| {
                let inbox = sample_mailbox("b2-in", owner, "INBOX", MailboxRole::Inbox);
                tx.upsert_mailbox(&inbox)?;
                let marker = SyncMarker {
                    uid_validity: validity,
                    uid_next: 10,
                    highest_modseq: None,
                };
                tx.set_mailbox_marker(&MailboxId::from_raw("b2-in"), Some(marker))?;
                let date = Timestamp::from_millis(500);
                tx.insert_message(&sample_message("msg-b1", owner), date)?;
                tx.add_appearance(&appearance("msg-b1", "b2-in", 1))?;
                tx.refresh_mailbox_counters(&MailboxId::from_raw("b2-in"))
            })
            .unwrap();
        account.id
    }
}
