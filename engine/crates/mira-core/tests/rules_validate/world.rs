//! Monde de test : comptes et boîtes connus, et règle valide de référence.

use crate::support_rules::probe::{cond, rule, text};
use mira_core::rules::validate::RuleWorld;
use mira_vocabulary::{AccountId, ConditionTest, MailboxId, Rule, RuleAction, RuleScope, TextOp};

/// Comptes et boîtes (boîte, compte) existants
pub struct FakeWorld {
    pub accounts: Vec<&'static str>,
    pub mailboxes: Vec<(&'static str, &'static str)>,
}

impl RuleWorld for FakeWorld {
    /// Vrai si le compte figure dans la liste
    fn account_exists(&self, account: &AccountId) -> bool {
        self.accounts.contains(&account.as_str())
    }

    /// Compte de la boîte, s'il la connaît
    fn mailbox_account(&self, mailbox: &MailboxId) -> Option<AccountId> {
        self.mailboxes
            .iter()
            .find(|(id, _)| *id == mailbox.as_str())
            .map(|(_, account)| AccountId::from_raw(*account))
    }
}

/// Monde de référence : acc-1 et acc-2, boîte mb-1 à acc-1, boîte mb-2 à acc-2
pub fn world() -> FakeWorld {
    FakeWorld {
        accounts: vec!["acc-1", "acc-2"],
        mailboxes: vec![("mb-1", "acc-1"), ("mb-2", "acc-2")],
    }
}

/// Règle correcte limitée à acc-1, qui range dans mb-1
pub fn valid_rule() -> Rule {
    let test = ConditionTest::Subject {
        matcher: text(TextOp::Contains, "facture"),
    };
    let actions = vec![RuleAction::MoveTo {
        mailbox: MailboxId::from_raw("mb-1"),
    }];
    let mut result = rule("r1", vec![vec![cond(test)]], actions);
    result.scope = RuleScope::Accounts(vec![AccountId::from_raw("acc-1")]);
    result
}

/// Règle valide dont l'unique action est remplacée
pub fn with_action(action: RuleAction) -> Rule {
    let mut rule = valid_rule();
    rule.actions = vec![action];
    rule
}
