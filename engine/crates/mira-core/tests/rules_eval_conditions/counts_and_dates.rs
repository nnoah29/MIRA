//! Conditions numériques : destinataires, dates et taille.

use crate::support_rules::fixtures::{DAY_MILLIS, NOW_MILLIS, base_context};
use crate::support_rules::probe::test_holds;
use mira_vocabulary::{Comparison, ConditionTest, Timestamp};

/// Le nombre de destinataires compte À, Cc et Cci
#[test]
fn recipient_count_compares_all_recipients() {
    let ctx = base_context();
    let by = |cmp, n| ConditionTest::RecipientCount { cmp, n };
    assert!(test_holds(by(Comparison::Equal, 3), &ctx));
    assert!(test_holds(by(Comparison::Greater, 2), &ctx));
    assert!(test_holds(by(Comparison::Less, 4), &ctx));
    assert!(!test_holds(by(Comparison::Greater, 3), &ctx));
}

/// `ReceivedBefore` : date strictement antérieure
#[test]
fn received_before_compares_message_date() {
    let ctx = base_context();
    let by = |millis| ConditionTest::ReceivedBefore {
        at: Timestamp::from_millis(millis),
    };
    assert!(test_holds(by(NOW_MILLIS), &ctx));
    assert!(!test_holds(by(NOW_MILLIS - 11 * DAY_MILLIS), &ctx));
}

/// `ReceivedAfter` : date strictement postérieure
#[test]
fn received_after_compares_message_date() {
    let ctx = base_context();
    let by = |millis| ConditionTest::ReceivedAfter {
        at: Timestamp::from_millis(millis),
    };
    assert!(test_holds(by(NOW_MILLIS - 11 * DAY_MILLIS), &ctx));
    assert!(!test_holds(by(NOW_MILLIS), &ctx));
}

/// `OlderThanDays` compare `now` à la date, et à la date interne à défaut
#[test]
fn older_than_days_uses_date_then_internal_date() {
    let mut ctx = base_context();
    assert!(test_holds(ConditionTest::OlderThanDays { days: 7 }, &ctx));
    assert!(!test_holds(ConditionTest::OlderThanDays { days: 30 }, &ctx));
    ctx.summary.date = None;
    assert!(!test_holds(ConditionTest::OlderThanDays { days: 7 }, &ctx));
    ctx.summary.internal_date = Some(Timestamp::from_millis(NOW_MILLIS - 20 * DAY_MILLIS));
    assert!(test_holds(ConditionTest::OlderThanDays { days: 7 }, &ctx));
}

/// `SizeCompare` compare la taille en octets
#[test]
fn size_compare_uses_bytes() {
    let ctx = base_context();
    let by = |cmp, bytes| ConditionTest::SizeCompare { cmp, bytes };
    assert!(test_holds(by(Comparison::Greater, 1_000), &ctx));
    assert!(test_holds(by(Comparison::Equal, 2_000), &ctx));
    assert!(!test_holds(by(Comparison::Less, 2_000), &ctx));
}
