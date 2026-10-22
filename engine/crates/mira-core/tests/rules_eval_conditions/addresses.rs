//! Conditions sur les expéditeurs, destinataires et adresses du compte.

use crate::support_rules::fixtures::base_context;
use crate::support_rules::probe::{test_holds, text};
use mira_vocabulary::{AddressField, ConditionTest, RecipientKind, TextOp};

/// L'expéditeur se filtre sur l'adresse, le nom et le domaine
#[test]
fn sender_matches_address_name_and_domain() {
    let ctx = base_context();
    let by = |field, op, value: &str| ConditionTest::Sender {
        field,
        matcher: text(op, value),
    };
    let address = by(AddressField::Address, TextOp::Equals, "bob@example.org");
    assert!(test_holds(address, &ctx));
    let name = by(AddressField::Name, TextOp::Contains, "durand");
    assert!(test_holds(name, &ctx));
    let domain = by(AddressField::Domain, TextOp::Equals, "example.org");
    assert!(test_holds(domain, &ctx));
    let other = by(AddressField::Domain, TextOp::Equals, "example.net");
    assert!(!test_holds(other, &ctx));
}

/// L'adresse de réponse se filtre comme l'expéditeur
#[test]
fn reply_to_matches_domain() {
    let ctx = base_context();
    let by = |value: &str| ConditionTest::ReplyTo {
        field: AddressField::Domain,
        matcher: text(TextOp::Equals, value),
    };
    assert!(test_holds(by("example.net"), &ctx));
    assert!(!test_holds(by("example.org"), &ctx));
}

/// Un destinataire se filtre selon son rôle
#[test]
fn recipient_matches_only_the_given_role() {
    let ctx = base_context();
    let by = |which, value: &str| ConditionTest::Recipient {
        which,
        field: AddressField::Address,
        matcher: text(TextOp::Equals, value),
    };
    assert!(test_holds(by(RecipientKind::To, "carol@example.org"), &ctx));
    assert!(test_holds(by(RecipientKind::Cc, "dave@example.org"), &ctx));
    assert!(!test_holds(
        by(RecipientKind::Cc, "carol@example.org"),
        &ctx
    ));
    assert!(!test_holds(
        by(RecipientKind::Bcc, "dave@example.org"),
        &ctx
    ));
}

/// `MeInTo` : une adresse du compte figure en À
#[test]
fn me_in_to_requires_own_address_in_to() {
    let mut ctx = base_context();
    assert!(test_holds(ConditionTest::MeInTo, &ctx));
    ctx.summary.to.remove(0);
    assert!(!test_holds(ConditionTest::MeInTo, &ctx));
}

/// `MeInCcOnly` : en Cc et pas en À
#[test]
fn me_in_cc_only_requires_cc_and_not_to() {
    let mut ctx = base_context();
    assert!(!test_holds(ConditionTest::MeInCcOnly, &ctx));
    ctx.summary.cc = vec![ctx.summary.to.remove(0)];
    assert!(test_holds(ConditionTest::MeInCcOnly, &ctx));
    ctx.summary.to.push(ctx.summary.cc[0].clone());
    assert!(!test_holds(ConditionTest::MeInCcOnly, &ctx));
}
