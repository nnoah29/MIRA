//! Amorçage du miroir local d'un monde de test : une boîte et des messages en-têtes seuls, indexés pour la recherche.

use mira_assembly::mira_vocabulary::{
    AccountId, AddressEntry, Appearance, ContentState, Flags, Mailbox, MailboxId, MailboxRole,
    MessageId, MessageSummary, MiraError, Timestamp,
};
use mira_assembly::profile::ProfilePaths;
use mira_assembly::storage::{Database, Tx};

use super::world::TestWorld;

/// Boîte de test sans repère de synchronisation.
fn mailbox_row(id: &MailboxId, account: &AccountId, name: &str, role: MailboxRole) -> Mailbox {
    Mailbox {
        id: id.clone(),
        account: account.clone(),
        name: name.to_string(),
        display_name: name.to_string(),
        delimiter: Some("/".to_string()),
        parent: None,
        role,
        sync_enabled: true,
        marker: None,
        total: 0,
        unread: 0,
    }
}

/// Message de test non lu, en-têtes seuls, d'expéditeur bob@example.org.
fn message_row(id: &MessageId, account: &AccountId, subject: &str) -> MessageSummary {
    let sender = AddressEntry {
        name: Some("Bob".to_string()),
        address: "bob@example.org".to_string(),
    };
    MessageSummary {
        id: id.clone(),
        account: account.clone(),
        thread: None,
        from: vec![sender],
        to: Vec::new(),
        cc: Vec::new(),
        bcc: Vec::new(),
        reply_to: Vec::new(),
        subject: subject.to_string(),
        date: Some(Timestamp::from_millis(5_000)),
        internal_date: Some(Timestamp::from_millis(5_100)),
        message_id_header: Some(format!("<{}@example.org>", id.as_str())),
        in_reply_to: None,
        references: Vec::new(),
        flags: Flags::default(),
        size: 1_234,
        preview: format!("Aperçu {subject}"),
        has_attachments: false,
        unsubscribe: None,
        content: ContentState::HeadersOnly,
        mailboxes: Vec::new(),
    }
}

/// Insère le message `index` de la boîte, son apparition (UID `index + 1`) et son entrée d'index de recherche.
fn seed_message(
    tx: &Tx<'_>,
    mailbox: &MailboxId,
    account: &AccountId,
    name: &str,
    index: usize,
    subject: &str,
) -> Result<(), MiraError> {
    let id = MessageId::from_raw(format!("{name}-{}", index + 1));
    let sorted_at = Timestamp::from_millis(1_000 * (index as i64 + 1));
    tx.insert_message(&message_row(&id, account, subject), sorted_at)?;
    tx.index_message(&id, subject, "Bob bob@example.org", "")?;
    tx.add_appearance(&Appearance {
        message: id,
        mailbox: mailbox.clone(),
        uid: index as u32 + 1,
        deleted: false,
        needs_revalidation: false,
    })
}

impl TestWorld {
    /// Ouvre la base du profil (le moteur reste ouvert : SQLite sérialise les écritures).
    pub(super) fn open_mirror(&self) -> Result<Database, MiraError> {
        let paths = ProfilePaths::resolve(&self.config.profile, &self.config.environment)?;
        Database::open(&paths.database)
    }

    /// Crée une boîte `name` du compte de test avec un message par sujet (UID 1, 2, ...), dates croissantes.
    pub fn seed_mailbox(
        &self,
        name: &str,
        role: MailboxRole,
        subjects: &[&str],
    ) -> Result<MailboxId, MiraError> {
        let account = self
            .account()
            .unwrap_or_else(|| AccountId::from_raw("acc-seed"));
        let mailbox = MailboxId::from_raw(format!("box-{name}"));
        self.open_mirror()?.write(|tx| {
            tx.upsert_mailbox(&mailbox_row(&mailbox, &account, name, role))?;
            for (index, subject) in subjects.iter().enumerate() {
                seed_message(tx, &mailbox, &account, name, index, subject)?;
            }
            tx.refresh_mailbox_counters(&mailbox)
        })?;
        Ok(mailbox)
    }
}
