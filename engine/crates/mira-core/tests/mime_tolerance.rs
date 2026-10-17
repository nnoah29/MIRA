#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Analyse MIME tolérante : messages mal formés, tronqués ou trop gros ne font jamais d'erreur.

use mira_core::mime::{attachment_bytes, parse_message};
use proptest::prelude::*;

/// Un message sans en-tête reste lisible comme un corps ou un objet vide
#[test]
fn message_without_headers_does_not_fail() {
    let parsed = parse_message(b"juste du texte sans en-tete\r\n");
    assert!(parsed.headers.subject.is_empty());
    let empty = parse_message(b"");
    assert!(empty.text_body.is_empty() && empty.attachments.is_empty());
}

/// Un multipart tronqué rend ce qui a pu être lu
#[test]
fn truncated_multipart_returns_what_it_can() {
    let raw = b"Subject: coupe\r\nContent-Type: multipart/mixed; boundary=\"b\"\r\n\r\n--b\r\nContent-Type: text/plain\r\n\r\nle debut du corps\r\n--b\r\nContent-Type: application/pdf\r\nContent-Transfer-Encoding: base64\r\n\r\nSGVsbG8";
    let parsed = parse_message(raw);
    assert_eq!(parsed.headers.subject, "coupe");
    assert!(parsed.text_body.contains("le debut du corps"));
}

/// Un encodage invalide est remplacé et non refusé
#[test]
fn invalid_encoding_is_replaced() {
    let mut raw = b"Subject: x\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n".to_vec();
    raw.extend_from_slice(&[b'a', 0xFF, 0xFE, b'b', b'\r', b'\n']);
    let parsed = parse_message(&raw);
    assert!(parsed.text_body.contains('a') && parsed.text_body.contains('b'));
}

/// Au-delà de 50 Mo, seuls les en-têtes sont lus et une pièce jointe non décrite est signalée
#[test]
fn oversized_message_keeps_headers_only() {
    let mut raw = b"Subject: enorme\r\nFrom: a@example.org\r\n\r\n".to_vec();
    raw.resize(51 * 1024 * 1024, b'x');
    let parsed = parse_message(&raw);
    assert_eq!(parsed.headers.subject, "enorme");
    assert!(parsed.text_body.is_empty() && parsed.html_body.is_none());
    assert_eq!(parsed.attachments.len(), 1);
    assert!(parsed.attachments[0].filename.is_none());
    assert!(attachment_bytes(&raw, 0).is_none());
}

proptest! {
    /// Aucune suite d'octets ne fait paniquer l'analyse
    #[test]
    fn arbitrary_bytes_never_panic(bytes in proptest::collection::vec(any::<u8>(), 0..2048)) {
        let parsed = parse_message(&bytes);
        let _ = parsed.attachments.len();
        let _ = attachment_bytes(&bytes, 0);
        let _ = mira_core::mime::parse_headers(&bytes);
    }
}
