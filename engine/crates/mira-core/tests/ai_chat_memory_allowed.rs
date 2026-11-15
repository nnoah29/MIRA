#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Consentement du rappel automatique : les souvenirs ne quittent l'appareil que si le type « mémoire » est autorisé.

use mira_core::ai::chat::memory_allowed;
use mira_vocabulary::ContentKind;

/// Hors de l'appareil, les souvenirs ne partent que si le type « mémoire » est autorisé
#[test]
fn memories_leave_the_device_only_when_the_kind_is_allowed() {
    assert!(memory_allowed(&[], true));
    assert!(memory_allowed(&[ContentKind::Memory], false));
    assert!(!memory_allowed(&[ContentKind::MailMessage], false));
}
