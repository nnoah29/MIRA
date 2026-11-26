#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Composition MIME : un nom ou un type de pièce jointe piégé ne peut pas injecter d'en-tête.

mod support_smtp;

use mira_core::smtp::compose::{ComposeAttachment, build_message};
use support_smtp::{sample_input, text_of};

/// Retourne les lignes d'en-tête et de parties du message produit
fn produced_lines(filename: &str, mime_type: &str) -> Vec<String> {
    let mut input = sample_input();
    input.attachments = vec![ComposeAttachment {
        filename: filename.to_string(),
        mime_type: mime_type.to_string(),
        data: b"contenu".to_vec(),
    }];
    let built = build_message(&input).expect("la composition doit réussir");
    text_of(&built).lines().map(str::to_string).collect()
}

/// Un saut de ligne dans le nom de la pièce jointe n'ouvre aucun nouvel en-tête
#[test]
fn line_break_in_attachment_name_cannot_inject_headers() {
    let lines = produced_lines("a.txt\r\nBcc: espion@example.org", "text/plain");
    assert!(lines.iter().any(|l| l.contains("a.txt")), "{lines:?}");
    assert!(!lines.iter().any(|l| l.starts_with("Bcc:")));
    assert!(!lines.iter().any(|l| l.starts_with("X-Evil:")));
}

/// Un saut de ligne dans le type MIME n'ouvre aucun nouvel en-tête
#[test]
fn line_break_in_attachment_type_cannot_inject_headers() {
    let lines = produced_lines("a.txt", "text/plain\r\nX-Evil: 1");
    assert!(lines.iter().any(|l| l.contains("a.txt")), "{lines:?}");
    assert!(!lines.iter().any(|l| l.starts_with("X-Evil:")));
}
