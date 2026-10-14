//! Objets de miroir de test : boîte, message en-têtes seuls et table des cinq messages de départ.

use mira_assembly::mira_vocabulary::{
    AccountId, AddressEntry, ContentState, Flags, Mailbox, MailboxId, MailboxRole, MessageId,
    MessageSummary, Timestamp,
};

/// Messages de départ (id, expéditeur, drapeaux) : m2 suivi, m3 et m4 lus, m4 et m5 de Carla.
pub fn message_rows() -> Vec<(&'static str, (&'static str, &'static str), Flags)> {
    let seen = Flags {
        seen: true,
        ..Flags::default()
    };
    let flagged = Flags {
        flagged: true,
        ..Flags::default()
    };
    let bob = ("Bob", "bob@example.org");
    let carla = ("Carla", "carla@example.org");
    vec![
        ("m1", bob, Flags::default()),
        ("m2", bob, flagged),
        ("m3", bob, seen),
        ("m4", carla, seen),
        ("m5", carla, Flags::default()),
    ]
}

/// Boîte de test d'un compte, sans repère de synchronisation.
pub fn mailbox(id: &str, account: &str, name: &str, role: MailboxRole) -> Mailbox {
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

/// Message de test : expéditeur et drapeaux choisis, en-têtes seuls.
pub fn message(id: &str, account: &str, sender: (&str, &str), flags: Flags) -> MessageSummary {
    let entry = |name: &str, address: &str| AddressEntry {
        name: Some(name.to_string()),
        address: address.to_string(),
    };
    MessageSummary {
        id: MessageId::from_raw(id),
        account: AccountId::from_raw(account),
        thread: None,
        from: vec![entry(sender.0, sender.1)],
        to: vec![entry("Alice", "alice@folders.test")],
        cc: Vec::new(),
        bcc: Vec::new(),
        reply_to: Vec::new(),
        subject: format!("Sujet {id}"),
        date: Some(Timestamp::from_millis(5_000)),
        internal_date: Some(Timestamp::from_millis(5_100)),
        message_id_header: Some(format!("<{id}@example.org>")),
        in_reply_to: None,
        references: Vec::new(),
        flags,
        size: 1_234,
        preview: format!("Aperçu {id}"),
        has_attachments: false,
        unsubscribe: None,
        content: ContentState::HeadersOnly,
        mailboxes: Vec::new(),
    }
}
