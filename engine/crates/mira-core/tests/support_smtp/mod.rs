#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]
//! Fabriques partagées par les tests SMTP : message à composer et lecture du résultat.

use std::time::Duration;

use mira_core::smtp::compose::{BuiltMessage, ComposeInput};
use mira_vocabulary::{AddressEntry, Timestamp};

pub const LIMIT: Duration = Duration::from_secs(20);

/// Entrée de composition type : alice écrit à bob, avec Cc et Bcc
pub fn sample_input() -> ComposeInput {
    let entry = |name: Option<&str>, address: &str| AddressEntry {
        name: name.map(str::to_string),
        address: address.to_string(),
    };
    ComposeInput {
        from: entry(Some("Alice"), "alice@example.org"),
        to: vec![entry(Some("Bob"), "bob@example.org")],
        cc: vec![entry(None, "carol@example.org")],
        bcc: vec![entry(None, "dave@example.org")],
        subject: "Bonjour".to_string(),
        date: Timestamp::from_millis(1_760_000_000_000),
        message_id: "<m1@mira.local>".to_string(),
        in_reply_to: None,
        references: Vec::new(),
        body_text: "Salut Bob\r\n".to_string(),
        body_html: None,
        attachments: Vec::new(),
    }
}

/// Texte du message composé, pour des vérifications d'en-têtes
pub fn text_of(message: &BuiltMessage) -> String {
    String::from_utf8_lossy(&message.raw).into_owned()
}
