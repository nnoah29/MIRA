#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Noms et types des pièces jointes nettoyés dès l'analyse (audit 8) : jamais de chemin ni de contrôle.

use mira_core::mime::parse_message;

/// Message à une pièce jointe dont le nom et le type sont fournis tels quels
fn with_attachment(content_type: &str, filename: &str) -> Vec<u8> {
    format!(
        "From: a@example.org\r\nSubject: PJ\r\nMIME-Version: 1.0\r\nContent-Type: multipart/mixed; boundary=\"m1\"\r\n\r\n--m1\r\nContent-Type: text/plain\r\n\r\ncorps\r\n--m1\r\nContent-Type: {content_type}\r\nContent-Disposition: attachment; filename=\"{filename}\"\r\nContent-Transfer-Encoding: base64\r\n\r\nSGVsbG8=\r\n--m1--\r\n"
    )
    .into_bytes()
}

/// Nom nettoyé de la première pièce jointe
fn cleaned_name(filename: &str) -> Option<String> {
    let parsed = parse_message(&with_attachment("application/pdf", filename));
    parsed.attachments[0].filename.clone()
}

/// Seul le dernier composant du chemin est gardé
#[test]
fn regression_path_components_are_dropped_from_attachment_names() {
    assert_eq!(cleaned_name("../../etc/passwd").as_deref(), Some("passwd"));
    assert_eq!(
        cleaned_name(r"..\\..\\evil.exe").as_deref(),
        Some("evil.exe")
    );
    assert_eq!(
        cleaned_name("/abs/olu/rapport.pdf").as_deref(),
        Some("rapport.pdf")
    );
}

/// Les noms réservés ou vides après nettoyage disparaissent
#[test]
fn regression_dot_names_and_empty_names_are_refused() {
    assert_eq!(cleaned_name(".."), None);
    assert_eq!(cleaned_name("."), None);
    assert_eq!(cleaned_name("a/"), None);
    assert_eq!(cleaned_name("   "), None);
}

/// Contrôles et inversions bidirectionnelles sont retirés, la longueur est bornée à 255 octets
#[test]
fn regression_controls_bidi_and_length_are_cleaned() {
    let tricky = "gp\u{202e}fdp.exe\u{1}x";
    assert_eq!(cleaned_name(tricky).as_deref(), Some("gpfdp.exex"));
    let long = format!("{}.pdf", "é".repeat(300));
    let name = cleaned_name(&long).unwrap();
    assert!(name.len() <= 255, "{}", name.len());
    assert!(name.is_char_boundary(name.len()));
}

/// Un type MIME mal formé retombe sur `application/octet-stream`
#[test]
fn regression_malformed_mime_types_fall_back_to_octet_stream() {
    let parsed = parse_message(&with_attachment("application/pd f<x>", "a.bin"));
    assert_eq!(parsed.attachments[0].mime_type, "application/octet-stream");
    let parsed = parse_message(&with_attachment("Application/PDF", "a.pdf"));
    assert_eq!(parsed.attachments[0].mime_type, "application/pdf");
}
