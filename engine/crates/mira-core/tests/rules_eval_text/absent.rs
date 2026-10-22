//! Champs absents : la condition positive est fausse, sa négation vraie.

use crate::support_rules::fixtures::base_context;
use crate::support_rules::probe::{cond, holds, not, text};
use mira_vocabulary::{AddressField, ConditionTest, TextOp};

/// Sans expéditeur, la condition est fausse et sa négation vraie
#[test]
fn missing_sender_is_false_and_negation_true() {
    let mut ctx = base_context();
    ctx.summary.from.clear();
    let test = ConditionTest::Sender {
        field: AddressField::Address,
        matcher: text(TextOp::Contains, "bob"),
    };
    assert!(!holds(cond(test.clone()), &ctx));
    assert!(holds(not(test), &ctx));
}

/// Sans nom d'affichage, un filtre sur le nom est faux
#[test]
fn missing_display_name_is_false() {
    let mut ctx = base_context();
    ctx.summary.from[0].name = None;
    let test = ConditionTest::Sender {
        field: AddressField::Name,
        matcher: text(TextOp::Contains, "bob"),
    };
    assert!(!holds(cond(test.clone()), &ctx));
    assert!(holds(not(test), &ctx));
}

/// Sans objet ni aperçu, les filtres sont faux et leurs négations vraies
#[test]
fn missing_subject_and_preview_are_false_and_negation_true() {
    let mut ctx = base_context();
    ctx.summary.subject.clear();
    ctx.summary.preview.clear();
    let subject = ConditionTest::Subject {
        matcher: text(TextOp::Contains, "facture"),
    };
    let preview = ConditionTest::Preview {
        matcher: text(TextOp::Contains, "reçu"),
    };
    assert!(!holds(cond(subject.clone()), &ctx));
    assert!(holds(not(subject), &ctx));
    assert!(!holds(cond(preview.clone()), &ctx));
    assert!(holds(not(preview), &ctx));
}
