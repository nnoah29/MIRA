//! Conditions sur les drapeaux et la nature du message.

use crate::support_rules::fixtures::base_context;
use crate::support_rules::probe::test_holds;
use mira_vocabulary::{ConditionTest, Unsubscribe};

/// `IsRead` suit le drapeau lu
#[test]
fn is_read_follows_seen_flag() {
    let mut ctx = base_context();
    assert!(!test_holds(ConditionTest::IsRead, &ctx));
    ctx.summary.flags.seen = true;
    assert!(test_holds(ConditionTest::IsRead, &ctx));
}

/// `IsFlagged` suit le drapeau étoile
#[test]
fn is_flagged_follows_flagged_flag() {
    let mut ctx = base_context();
    assert!(!test_holds(ConditionTest::IsFlagged, &ctx));
    ctx.summary.flags.flagged = true;
    assert!(test_holds(ConditionTest::IsFlagged, &ctx));
}

/// `IsAnswered` suit le drapeau répondu
#[test]
fn is_answered_follows_answered_flag() {
    let mut ctx = base_context();
    assert!(!test_holds(ConditionTest::IsAnswered, &ctx));
    ctx.summary.flags.answered = true;
    assert!(test_holds(ConditionTest::IsAnswered, &ctx));
}

/// `IsForwarded` suit le drapeau transféré
#[test]
fn is_forwarded_follows_forwarded_flag() {
    let mut ctx = base_context();
    assert!(!test_holds(ConditionTest::IsForwarded, &ctx));
    ctx.summary.flags.forwarded = true;
    assert!(test_holds(ConditionTest::IsForwarded, &ctx));
}

/// `IsNewsletter` : le message annonce un désabonnement
#[test]
fn is_newsletter_requires_unsubscribe() {
    let mut ctx = base_context();
    assert!(!test_holds(ConditionTest::IsNewsletter, &ctx));
    ctx.summary.unsubscribe = Some(Unsubscribe {
        https_url: None,
        mailto: Some("mailto:stop@example.org".to_string()),
        one_click: false,
    });
    assert!(test_holds(ConditionTest::IsNewsletter, &ctx));
}

/// `IsReply` : le message répond à un autre
#[test]
fn is_reply_requires_in_reply_to() {
    let mut ctx = base_context();
    assert!(!test_holds(ConditionTest::IsReply, &ctx));
    ctx.summary.in_reply_to = Some("<m0@example.org>".to_string());
    assert!(test_holds(ConditionTest::IsReply, &ctx));
}
