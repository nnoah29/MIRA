//! Conditions d'appartenance : fil, boîte, compte, et inversion.

use crate::support_rules::fixtures::base_context;
use crate::support_rules::probe::{holds, not, test_holds};
use mira_vocabulary::{AccountId, ConditionTest, MailboxId};

/// `InExistingThread` suit l'indication du contexte
#[test]
fn in_existing_thread_follows_context() {
    let mut ctx = base_context();
    assert!(!test_holds(ConditionTest::InExistingThread, &ctx));
    ctx.in_existing_thread = true;
    assert!(test_holds(ConditionTest::InExistingThread, &ctx));
}

/// `InMailbox` : le message figure dans la boîte
#[test]
fn in_mailbox_checks_message_mailboxes() {
    let ctx = base_context();
    let by = |id: &str| ConditionTest::InMailbox {
        mailbox: MailboxId::from_raw(id),
    };
    assert!(test_holds(by("box-in"), &ctx));
    assert!(!test_holds(by("box-other"), &ctx));
}

/// `FromAccount` : le message appartient au compte
#[test]
fn from_account_checks_account() {
    let ctx = base_context();
    let by = |id: &str| ConditionTest::FromAccount {
        account: AccountId::from_raw(id),
    };
    assert!(test_holds(by("acc-1"), &ctx));
    assert!(!test_holds(by("acc-2"), &ctx));
}

/// L'inversion retourne le résultat de la condition
#[test]
fn negation_inverts_result() {
    let mut ctx = base_context();
    assert!(holds(not(ConditionTest::IsRead), &ctx));
    ctx.summary.flags.seen = true;
    assert!(!holds(not(ConditionTest::IsRead), &ctx));
    assert!(holds(not(ConditionTest::IsFlagged), &ctx));
}
