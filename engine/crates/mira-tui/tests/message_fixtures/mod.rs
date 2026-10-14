//! Fabriques de messages, de pages et de rendus partagées par les tests de la liste de messages.
#![allow(dead_code)]

use mira_assembly::mira_vocabulary::{
    AccountId, AddressEntry, ContentState, Cursor, Flags, MailboxId, MessageId, MessageSummary,
    Page, Timestamp, ViewRef,
};

/// Vue de la boîte de réception du compte de test.
pub fn inbox_view() -> ViewRef {
    ViewRef::Mailbox {
        mailbox: MailboxId::from_raw("a1-INBOX"),
    }
}

/// Message de test avec cet identifiant, cet objet et cet expéditeur.
pub fn summary(id: &str, subject: &str, sender: &str) -> MessageSummary {
    MessageSummary {
        id: MessageId::from_raw(id),
        account: AccountId::from_raw("a1"),
        thread: None,
        from: vec![AddressEntry {
            name: Some(sender.to_string()),
            address: "bob@example.org".to_string(),
        }],
        to: Vec::new(),
        cc: Vec::new(),
        bcc: Vec::new(),
        reply_to: Vec::new(),
        subject: subject.to_string(),
        date: Some(Timestamp::from_millis(86_400_000 * 20 + 3_600_000 * 14)),
        internal_date: None,
        message_id_header: None,
        in_reply_to: None,
        references: Vec::new(),
        flags: Flags::default(),
        size: 100,
        preview: String::new(),
        has_attachments: false,
        unsubscribe: None,
        content: ContentState::HeadersOnly,
        mailboxes: Vec::new(),
    }
}

/// Fabrique `count` messages numérotés.
pub fn numbered(count: usize) -> Vec<MessageSummary> {
    (0..count)
        .map(|n| summary(&format!("m{n}"), &format!("objet {n}"), "Bob"))
        .collect()
}

/// Page de `count` messages numérotés à partir de `start`, avec un curseur suivant éventuel.
pub fn page(start: usize, count: usize, next: Option<&str>) -> Page<MessageSummary> {
    let items = (start..start + count)
        .map(|n| summary(&format!("m{n}"), &format!("objet {n}"), "Bob"))
        .collect();
    Page {
        items,
        next_cursor: next.map(|value| Cursor(value.to_string())),
    }
}
