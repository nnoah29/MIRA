#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Analyse MIME : messages simples, alternatifs, pièces jointes et ressources incorporées.

use mira_core::mime::{attachment_bytes, parse_message};

const SIMPLE: &[u8] = b"From: Alice Martin <alice@example.org>\r\nTo: bob@example.org\r\nSubject: Bonjour\r\nMessage-ID: <abc@mira>\r\nDate: Tue, 14 Nov 2023 22:13:20 +0000\r\n\r\nSalut Bob,\r\nA demain.\r\n";

const ALTERNATIVE: &[u8] = b"From: a@example.org\r\nSubject: Alt\r\nMIME-Version: 1.0\r\nContent-Type: multipart/alternative; boundary=\"b1\"\r\n\r\n--b1\r\nContent-Type: text/plain; charset=utf-8\r\n\r\nversion texte\r\n--b1\r\nContent-Type: text/html; charset=utf-8\r\n\r\n<p>version <b>html</b></p>\r\n--b1--\r\n";

const WITH_ATTACHMENT: &[u8] = b"From: a@example.org\r\nSubject: PJ\r\nMIME-Version: 1.0\r\nContent-Type: multipart/mixed; boundary=\"m1\"\r\n\r\n--m1\r\nContent-Type: text/plain\r\n\r\ncorps\r\n--m1\r\nContent-Type: application/pdf; name=\"rapport.pdf\"\r\nContent-Disposition: attachment; filename=\"rapport.pdf\"\r\nContent-Transfer-Encoding: base64\r\n\r\nSGVsbG8gUERG\r\n--m1--\r\n";

const WITH_INLINE: &[u8] = b"From: a@example.org\r\nSubject: Image\r\nMIME-Version: 1.0\r\nContent-Type: multipart/related; boundary=\"r1\"\r\n\r\n--r1\r\nContent-Type: text/html; charset=utf-8\r\n\r\n<img src=\"cid:logo@mira\">\r\n--r1\r\nContent-Type: image/png\r\nContent-ID: <logo@mira>\r\nContent-Disposition: inline; filename=\"logo.png\"\r\nContent-Transfer-Encoding: base64\r\n\r\niVBORw0KGgo=\r\n--r1--\r\n";

/// Un message simple donne ses en-têtes et son corps texte, sans HTML ni pièce jointe
#[test]
fn simple_message_has_headers_and_text() {
    let parsed = parse_message(SIMPLE);
    assert_eq!(parsed.headers.subject, "Bonjour");
    assert_eq!(parsed.headers.from[0].address, "alice@example.org");
    assert_eq!(parsed.headers.from[0].name.as_deref(), Some("Alice Martin"));
    assert_eq!(parsed.headers.message_id.as_deref(), Some("abc@mira"));
    assert_eq!(parsed.headers.date.unwrap().as_millis(), 1_700_000_000_000);
    assert!(parsed.text_body.contains("Salut Bob"));
    assert!(parsed.html_body.is_none() && parsed.attachments.is_empty());
}

/// Un multipart/alternative donne le texte et l'HTML séparément
#[test]
fn alternative_gives_text_and_html() {
    let parsed = parse_message(ALTERNATIVE);
    assert_eq!(parsed.text_body.trim(), "version texte");
    let html = parsed.html_body.unwrap();
    assert!(html.contains("<b>html</b>"), "{html}");
    assert!(parsed.attachments.is_empty());
}

/// Une pièce jointe est décrite et son contenu décodé est récupérable par son index
#[test]
fn attachment_is_described_and_extractable() {
    let parsed = parse_message(WITH_ATTACHMENT);
    assert_eq!(parsed.text_body.trim(), "corps");
    let attachment = &parsed.attachments[0];
    assert_eq!(attachment.filename.as_deref(), Some("rapport.pdf"));
    assert_eq!(attachment.mime_type, "application/pdf");
    assert_eq!(attachment.size, 9);
    assert!(!attachment.inline && attachment.content_id.is_none());
    let bytes = attachment_bytes(WITH_ATTACHMENT, attachment.part_index).unwrap();
    assert_eq!(bytes, b"Hello PDF");
}

/// Une ressource incorporée porte son Content-ID sans chevrons et reste « inline »
#[test]
fn inline_resource_keeps_its_content_id() {
    let parsed = parse_message(WITH_INLINE);
    assert!(parsed.html_body.unwrap().contains("cid:logo@mira"));
    let resource = &parsed.attachments[0];
    assert_eq!(resource.content_id.as_deref(), Some("logo@mira"));
    assert!(resource.inline);
    assert_eq!(resource.mime_type, "image/png");
}

/// Un index inconnu ne donne aucun contenu
#[test]
fn unknown_part_index_gives_none() {
    assert!(attachment_bytes(WITH_ATTACHMENT, 99).is_none());
    assert!(attachment_bytes(b"", 0).is_none());
}
