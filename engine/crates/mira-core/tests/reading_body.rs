#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Lectures d'un message, de son corps, de sa source et de ses pièces jointes, sans réseau (S6).

mod support;
mod support_mirror;
mod support_reading;

use mira_core::reading::{BodyView, get_body, get_message, get_raw_source, list_attachments};
use mira_vocabulary::{ErrorCategory, MailboxId, MessageId};
use support_reading::bodies::{attachment, database_with_bodies};

/// S6 : hors ligne, le corps est rendu s'il est présent, sinon l'erreur dédiée sans téléchargement
#[test]
fn scenario_s6_read_offline_body_present_or_unavailable() {
    let db = database_with_bodies();
    let body = get_body(&db, &MessageId::from_raw("full")).unwrap();
    let expected = BodyView {
        text: "Bonjour Alice".to_string(),
        html_available: true,
    };
    assert_eq!(body, expected);
    for id in ["headers", "stale"] {
        let error = get_body(&db, &MessageId::from_raw(id)).unwrap_err();
        assert_eq!(error.code, "incompatible_state.body_unavailable_offline");
        assert_eq!(error.category, ErrorCategory::IncompatibleState);
        assert!(!error.retryable);
        assert_eq!(error.context.message, Some(MessageId::from_raw(id)));
    }
}

/// Un corps sans version HTML l'indique
#[test]
fn text_only_body_reports_no_html() {
    let db = database_with_bodies();
    let body = get_body(&db, &MessageId::from_raw("plain")).unwrap();
    assert_eq!(body.text, "Texte seul");
    assert!(!body.html_available);
}

/// La source brute est rendue telle quelle si elle est stockée, sinon même erreur que le corps
#[test]
fn raw_source_is_returned_or_unavailable() {
    let db = database_with_bodies();
    let raw = get_raw_source(&db, &MessageId::from_raw("full")).unwrap();
    assert_eq!(raw, b"Subject: Sujet full\r\n\r\nBonjour Alice\r\n");
    let error = get_raw_source(&db, &MessageId::from_raw("plain")).unwrap_err();
    assert_eq!(error.code, "incompatible_state.body_unavailable_offline");
}

/// Un message inconnu est « not_found.message » pour toutes les lectures
#[test]
fn unknown_message_is_not_found_everywhere() {
    let db = database_with_bodies();
    let id = MessageId::from_raw("nope");
    assert_eq!(get_message(&db, &id).unwrap_err().code, "not_found.message");
    assert_eq!(get_body(&db, &id).unwrap_err().code, "not_found.message");
    assert_eq!(
        get_raw_source(&db, &id).unwrap_err().code,
        "not_found.message"
    );
    assert_eq!(
        list_attachments(&db, &id).unwrap_err().code,
        "not_found.message"
    );
}

/// Le résumé d'un message porte ses boîtes visibles
#[test]
fn get_message_returns_summary_with_mailboxes() {
    let db = database_with_bodies();
    let message = get_message(&db, &MessageId::from_raw("full")).unwrap();
    assert_eq!(message.subject, "Sujet full");
    assert_eq!(message.mailboxes, [MailboxId::from_raw("box-1")]);
}

/// Les pièces jointes sont listées dans l'ordre des parties
#[test]
fn attachments_are_listed_in_part_order() {
    let db = database_with_bodies();
    db.write(|tx| {
        tx.insert_attachment(&attachment("att-b", "full", "b.pdf"), 2)?;
        tx.insert_attachment(&attachment("att-a", "full", "a.pdf"), 1)
    })
    .unwrap();
    let listed = list_attachments(&db, &MessageId::from_raw("full")).unwrap();
    let names: Vec<_> = listed
        .iter()
        .filter_map(|a| a.filename.as_deref())
        .collect();
    assert_eq!(names, ["a.pdf", "b.pdf"]);
    assert!(
        list_attachments(&db, &MessageId::from_raw("plain"))
            .unwrap()
            .is_empty()
    );
}
