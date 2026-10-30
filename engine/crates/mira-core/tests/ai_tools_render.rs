#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Rendu d'un message en une ligne pour le modèle : identifiant, expéditeur, sujet, aperçu coupé.

mod support;
mod support_mirror;

use mira_core::ai::tools::render::mail_line;
use support_mirror::fixtures::sample_message;

/// La ligne d'un message porte son identifiant et un aperçu coupé à 200 caractères
#[test]
fn mail_line_cuts_preview_at_200_chars() {
    let mut message = sample_message("m1", "acc-1");
    message.preview = "é".repeat(300);
    let line = mail_line(&message);
    assert!(line.starts_with("id=m1 | de: Bob <bob@example.org> | "));
    assert!(line.contains("Sujet m1"));
    assert!(line.ends_with(&"é".repeat(200)));
}
