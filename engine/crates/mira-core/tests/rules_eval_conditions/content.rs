//! Conditions sur l'objet, l'aperçu et les pièces jointes.

use crate::support_rules::fixtures::base_context;
use crate::support_rules::probe::{test_holds, text};
use mira_vocabulary::{ConditionTest, TextOp};

/// L'objet se filtre par texte
#[test]
fn subject_matches_text() {
    let ctx = base_context();
    let by = |value: &str| ConditionTest::Subject {
        matcher: text(TextOp::StartsWith, value),
    };
    assert!(test_holds(by("facture"), &ctx));
    assert!(!test_holds(by("été"), &ctx));
}

/// L'aperçu se filtre par texte
#[test]
fn preview_matches_text() {
    let ctx = base_context();
    let by = |value: &str| ConditionTest::Preview {
        matcher: text(TextOp::Contains, value),
    };
    assert!(test_holds(by("REÇU"), &ctx));
    assert!(!test_holds(by("devis"), &ctx));
}

/// `HasAttachment` suit la présence de pièces jointes
#[test]
fn has_attachment_follows_attachments() {
    let mut ctx = base_context();
    assert!(test_holds(ConditionTest::HasAttachment, &ctx));
    ctx.attachments.clear();
    assert!(!test_holds(ConditionTest::HasAttachment, &ctx));
}

/// Le nom d'une pièce jointe se filtre par texte
#[test]
fn attachment_name_matches_any_attachment() {
    let mut ctx = base_context();
    let by = |value: &str| ConditionTest::AttachmentName {
        matcher: text(TextOp::EndsWith, value),
    };
    assert!(test_holds(by(".PDF"), &ctx));
    assert!(!test_holds(by(".zip"), &ctx));
    ctx.attachments.clear();
    assert!(!test_holds(by(".pdf"), &ctx));
}

/// Le type d'une pièce jointe se filtre par texte
#[test]
fn attachment_type_matches_media_type() {
    let ctx = base_context();
    let by = |value: &str| ConditionTest::AttachmentType {
        matcher: text(TextOp::Equals, value),
    };
    assert!(test_holds(by("application/pdf"), &ctx));
    assert!(!test_holds(by("image/png"), &ctx));
}
