//! Exemples de conditions numériques, de dates et de pièces jointes.

use mira_vocabulary::{Comparison, ConditionTest, Timestamp};

/// Liste une instance de chaque condition sur les nombres, dates et tailles.
pub fn numeric_conditions() -> Vec<ConditionTest> {
    vec![
        ConditionTest::MeInTo,
        ConditionTest::MeInCcOnly,
        ConditionTest::RecipientCount {
            cmp: Comparison::Greater,
            n: 3,
        },
        ConditionTest::ReceivedBefore {
            at: Timestamp::from_millis(10),
        },
        ConditionTest::ReceivedAfter {
            at: Timestamp::from_millis(20),
        },
        ConditionTest::OlderThanDays { days: 7 },
        ConditionTest::SizeCompare {
            cmp: Comparison::Less,
            bytes: 1024,
        },
        ConditionTest::HasAttachment,
    ]
}
