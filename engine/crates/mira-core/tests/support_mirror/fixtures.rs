//! Objets de miroir de test : boîte, message, apparition.

use mira_vocabulary::{
    AccountId, AddressEntry, Appearance, ContentState, Flags, Mailbox, MailboxId, MailboxRole,
    MessageId, MessageSummary, Timestamp,
};

/// Boîte de test d'un compte, sans repère ni compteur
pub fn sample_mailbox(id: &str, account: &str, name: &str, role: MailboxRole) -> Mailbox {
    Mailbox {
        id: MailboxId::from_raw(id),
        account: AccountId::from_raw(account),
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

/// Message de test d'un compte, non lu, en-têtes seuls, sans boîte
pub fn sample_message(id: &str, account: &str) -> MessageSummary {
    let entry = |name: &str, address: &str| AddressEntry {
        name: Some(name.to_string()),
        address: address.to_string(),
    };
    MessageSummary {
        id: MessageId::from_raw(id),
        account: AccountId::from_raw(account),
        thread: None,
        from: vec![entry("Bob", "bob@example.org")],
        to: vec![entry("Alice", "alice@example.org")],
        cc: Vec::new(),
        bcc: Vec::new(),
        reply_to: Vec::new(),
        subject: format!("Sujet {id}"),
        date: Some(Timestamp::from_millis(5_000)),
        internal_date: Some(Timestamp::from_millis(5_100)),
        message_id_header: Some(format!("<{id}@example.org>")),
        in_reply_to: None,
        references: vec!["<root@example.org>".to_string()],
        flags: Flags::default(),
        size: 1_234,
        preview: format!("Aperçu {id}"),
        has_attachments: false,
        unsubscribe: None,
        content: ContentState::HeadersOnly,
        mailboxes: Vec::new(),
    }
}

/// Apparition simple d'un message dans une boîte sous un UID
pub fn appearance(message: &str, mailbox: &str, uid: u32) -> Appearance {
    Appearance {
        message: MessageId::from_raw(message),
        mailbox: MailboxId::from_raw(mailbox),
        uid,
        deleted: false,
        needs_revalidation: false,
    }
}
