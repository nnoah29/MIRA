//! Contexte de message de référence pour les tests de l'évaluateur.

use mira_core::rules::context::{AttachmentInfo, MessageContext};
use mira_vocabulary::{
    AccountId, AddressEntry, ContentState, EmailAddress, Flags, MailboxId, MessageId,
    MessageSummary, Timestamp,
};

/// Instant d'évaluation des tests
pub const NOW_MILLIS: i64 = 1_000_000_000_000;

/// Millisecondes dans une journée
pub const DAY_MILLIS: i64 = 86_400_000;

/// Adresse avec nom d'affichage
pub fn named(name: &str, address: &str) -> AddressEntry {
    AddressEntry {
        name: Some(name.to_string()),
        address: address.to_string(),
    }
}

/// Message de référence : de Bob à main@ et carol@, copie à dave@, reçu 10 jours avant `NOW_MILLIS`
pub fn base_summary() -> MessageSummary {
    MessageSummary {
        id: MessageId::from_raw("msg-1"),
        account: AccountId::from_raw("acc-1"),
        thread: None,
        from: vec![named("Bob Durand", "bob@example.org")],
        to: vec![
            named("Moi", "main@example.org"),
            named("Carol", "carol@example.org"),
        ],
        cc: vec![named("Dave", "dave@example.org")],
        bcc: Vec::new(),
        reply_to: vec![named("Support", "support@example.net")],
        subject: "Facture Été".to_string(),
        date: Some(Timestamp::from_millis(NOW_MILLIS - 10 * DAY_MILLIS)),
        internal_date: None,
        message_id_header: Some("<m1@example.org>".to_string()),
        in_reply_to: None,
        references: Vec::new(),
        flags: Flags::default(),
        size: 2_000,
        preview: "Votre reçu est joint".to_string(),
        has_attachments: false,
        unsubscribe: None,
        content: ContentState::HeadersOnly,
        mailboxes: vec![MailboxId::from_raw("box-in")],
    }
}

/// Contexte de référence : le compte possède main@example.org, une pièce jointe PDF
pub fn base_context() -> MessageContext {
    MessageContext {
        summary: base_summary(),
        own_addresses: vec![EmailAddress::parse("main@example.org").unwrap()],
        attachments: vec![AttachmentInfo {
            name: "Facture-2026.pdf".to_string(),
            media_type: "application/pdf".to_string(),
        }],
        in_existing_thread: false,
        now: Timestamp::from_millis(NOW_MILLIS),
    }
}
