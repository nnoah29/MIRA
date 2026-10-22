//! Conditions qui désignent une boîte ou un compte : la cible doit exister et relever de la portée.

use crate::support_rules::probe::{cond, not};
use crate::world::{valid_rule, world};
use mira_core::rules::validate::{RuleStatus, rule_status, validate_rule};
use mira_vocabulary::{
    AccountId, Condition, ConditionTest, MailboxId, Rule, RuleAction, RuleScope,
};

/// Règle sans action de boîte, sur la portée donnée, dont l'unique condition est remplacée
fn with_condition(scope: RuleScope, condition: Condition) -> Rule {
    let mut rule = valid_rule();
    rule.actions = vec![RuleAction::Trash];
    rule.scope = scope;
    rule.alternatives[0].conditions = vec![condition];
    rule
}

/// Conditions « dans la boîte » et « reçu sur le compte » visant les cibles données
fn targets(mailbox: &str, account: &str) -> [ConditionTest; 2] {
    [
        ConditionTest::InMailbox {
            mailbox: MailboxId::from_raw(mailbox),
        },
        ConditionTest::FromAccount {
            account: AccountId::from_raw(account),
        },
    ]
}

/// Une boîte ou un compte inexistant est refusé, condition inversée comprise, et rend la règle invalide
#[test]
fn unknown_target_is_refused_and_invalid() {
    for test in targets("mb-9", "acc-9") {
        for condition in [cond(test.clone()), not(test)] {
            let rule = with_condition(RuleScope::AllAccounts, condition);
            let error = validate_rule(&rule, &world()).unwrap_err();
            assert_eq!(error.code, "invalid_input.parameter");
            assert!(!error.message.contains("9"));
            let status = rule_status(&rule, &world());
            assert!(matches!(status, RuleStatus::Invalid { .. }));
        }
    }
}

/// Des cibles existantes sont acceptées sur tous les comptes
#[test]
fn known_targets_are_accepted_on_all_accounts() {
    for test in targets("mb-2", "acc-2") {
        let rule = with_condition(RuleScope::AllAccounts, not(test));
        assert!(validate_rule(&rule, &world()).is_ok());
        assert_eq!(rule_status(&rule, &world()), RuleStatus::Valid);
    }
}

/// Une cible qui relève d'un compte hors de la portée est refusée
#[test]
fn target_outside_the_scope_is_refused() {
    let scope = RuleScope::Accounts(vec![AccountId::from_raw("acc-1")]);
    for test in targets("mb-2", "acc-2") {
        let rule = with_condition(scope.clone(), not(test));
        assert!(validate_rule(&rule, &world()).is_err());
    }
    for test in targets("mb-1", "acc-1") {
        let rule = with_condition(scope.clone(), cond(test));
        assert!(validate_rule(&rule, &world()).is_ok());
    }
}
