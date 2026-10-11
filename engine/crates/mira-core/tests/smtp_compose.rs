#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Composition MIME (10.8) : en-têtes, enveloppe, Bcc, corps, pièces jointes et fils.

mod support_smtp;

use mira_core::smtp::compose::{ComposeAttachment, build_message};
use support_smtp::{sample_input, text_of};

/// Les en-têtes attendus sont présents et les Bcc restent hors des en-têtes
#[test]
fn headers_are_present_and_bcc_stays_in_the_envelope_only() {
    let built = build_message(&sample_input()).unwrap();
    let text = text_of(&built);
    assert!(
        text.contains("From: \"Alice\" <alice@example.org>"),
        "{text}"
    );
    assert!(text.contains("To: \"Bob\" <bob@example.org>"), "{text}");
    assert!(text.contains("Cc: <carol@example.org>"), "{text}");
    assert!(text.contains("Subject: Bonjour"), "{text}");
    assert!(text.contains("Message-ID: <m1@mira.local>"), "{text}");
    assert!(
        text.contains("Date: Thu, 9 Oct 2025 08:53:20 +0000"),
        "{text}"
    );
    assert!(!text.contains("dave@example.org"));
    assert!(!text.to_lowercase().contains("bcc"));
    assert_eq!(built.message_id, "<m1@mira.local>");
    assert_eq!(built.envelope_from, "alice@example.org");
    let expected = ["bob@example.org", "carol@example.org", "dave@example.org"];
    assert_eq!(built.envelope_recipients, expected);
}

/// Texte seul, texte et HTML, puis pièce jointe donnent la structure MIME attendue
#[test]
fn body_structure_follows_the_parts_provided() {
    let mut input = sample_input();
    let plain = text_of(&build_message(&input).unwrap());
    assert!(plain.contains("Content-Type: text/plain"), "{plain}");
    assert!(!plain.contains("multipart"));
    input.body_html = Some("<p>Salut Bob</p>".to_string());
    let alternative = text_of(&build_message(&input).unwrap());
    assert!(
        alternative.contains("multipart/alternative"),
        "{alternative}"
    );
    assert!(alternative.contains("text/html"));
    input.attachments.push(ComposeAttachment {
        filename: "note.txt".to_string(),
        mime_type: "text/plain".to_string(),
        data: b"contenu joint".to_vec(),
    });
    let mixed = text_of(&build_message(&input).unwrap());
    assert!(mixed.contains("multipart/mixed"), "{mixed}");
    assert!(mixed.contains("note.txt"));
}

/// In-Reply-To et References reprennent les identifiants fournis
#[test]
fn reply_headers_carry_the_threading_identifiers() {
    let mut input = sample_input();
    input.in_reply_to = Some("<parent@mira.local>".to_string());
    input.references = vec![
        "root@mira.local".to_string(),
        "<parent@mira.local>".to_string(),
    ];
    let text = text_of(&build_message(&input).unwrap());
    assert!(text.contains("In-Reply-To: <parent@mira.local>"), "{text}");
    assert!(
        text.contains("References: <root@mira.local> <parent@mira.local>"),
        "{text}"
    );
}

/// Un sujet et un corps non ASCII sont encodés sans perte
#[test]
fn non_ascii_subject_and_body_are_encoded() {
    let mut input = sample_input();
    input.subject = "Été à Genève".to_string();
    input.body_text = "Voilà le résumé\r\n".to_string();
    let text = text_of(&build_message(&input).unwrap());
    assert!(text.contains("Subject: =?utf-8?"), "{text}");
    assert!(!text.contains("Été"));
    assert!(text.contains("charset=\"utf-8\""), "{text}");
}
