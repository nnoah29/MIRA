//! Exemples de conditions sur les drapeaux, la nature et l'emplacement.

use mira_vocabulary::{AccountId, ConditionTest, MailboxId};

/// Liste une instance de chaque condition sur l'état et l'emplacement.
pub fn state_conditions() -> Vec<ConditionTest> {
    vec![
        ConditionTest::IsRead,
        ConditionTest::IsFlagged,
        ConditionTest::IsAnswered,
        ConditionTest::IsForwarded,
        ConditionTest::IsNewsletter,
        ConditionTest::IsReply,
        ConditionTest::InExistingThread,
        ConditionTest::InMailbox {
            mailbox: MailboxId::from_raw("mb-1"),
        },
        ConditionTest::FromAccount {
            account: AccountId::from_raw("acc-1"),
        },
    ]
}
