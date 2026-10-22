#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Domaine « rule » par la façade : création, liste, idempotence, modification, activation, ordre, statut lu.

mod support;
mod support_api;
mod support_imap;
mod support_mirror;
mod support_rule_api;
mod support_write;

use mira_core::api::{AccountRequest, Request, RuleRequest};
use mira_core::rules::validate::RuleStatus;
use mira_vocabulary::{MailboxId, RuleId, RuleScope};
use support_imap::within;
use support_rule_api::calls::{create, list, listed_ids, rule_call, view_of};
use support_rule_api::samples::{changed_rules, move_rule, only, subject_rule};
use support_rule_api::writes::{create_named, delete, reorder, set_enabled, update};
use support_write::fixture::Fixture;

/// Créer puis lister : identifiants attribués par le moteur, ordre de création, statut valide, `RuleChanged` émis
#[tokio::test]
async fn create_then_list_returns_rules_in_order_with_status() {
    within(async {
        let f = Fixture::start().await;
        let all = subject_rule("Factures", "Facture", RuleScope::AllAccounts);
        let first = create(f.engine(), all, "k-1").await.unwrap();
        let mine = subject_rule("Lettres", "Lettre", only(&f.account));
        let second = create(f.engine(), mine, "k-2").await.unwrap();
        assert_ne!(first.rule.id, second.rule.id);
        assert_ne!(first.rule.id.as_str(), "client-id");
        assert_eq!(first.status, RuleStatus::Valid);
        let listed = list(f.engine()).await;
        assert_eq!(listed, vec![first.clone(), second.clone()]);
        let get = RuleRequest::GetRule {
            rule: second.rule.id.clone(),
        };
        assert_eq!(view_of(rule_call(f.engine(), get).await).unwrap(), second);
        let ids = [first.rule.id.as_str(), second.rule.id.as_str()];
        assert_eq!(changed_rules(f.engine()).await, ids);
    })
    .await;
}

/// La même clé rejouée rend la même règle sans doublon ni nouvel événement ; une autre commande sous cette clé est refusée
#[tokio::test]
async fn same_idempotency_key_returns_the_same_rule() {
    within(async {
        let f = Fixture::start().await;
        let rule = subject_rule("Factures", "Facture", RuleScope::AllAccounts);
        let first = create(f.engine(), rule.clone(), "k-1").await.unwrap();
        let again = create(f.engine(), rule, "k-1").await.unwrap();
        assert_eq!(again, first);
        assert_eq!(list(f.engine()).await.len(), 1);
        let other = subject_rule("Autre", "Devis", RuleScope::AllAccounts);
        let conflict = create(f.engine(), other, "k-1").await.unwrap_err();
        assert_eq!(conflict, "invalid_input.idempotency_conflict");
        delete(f.engine(), &first.rule.id, "k-del").await.unwrap();
        delete(f.engine(), &first.rule.id, "k-del").await.unwrap();
        assert!(list(f.engine()).await.is_empty());
        let unknown = delete(f.engine(), &first.rule.id, "k-del-2").await;
        assert_eq!(unknown.unwrap_err(), "not_found.rule");
        let id = first.rule.id.as_str();
        assert_eq!(changed_rules(f.engine()).await, [id, id]);
    })
    .await;
}

/// Modifier remplace la définition en gardant la position ; une règle inconnue donne `not_found.rule`
#[tokio::test]
async fn update_replaces_the_definition_in_place() {
    within(async {
        let f = Fixture::start().await;
        let ids = create_named(f.engine(), &["a", "b"]).await;
        let mut changed = subject_rule("A2", "Devis", RuleScope::AllAccounts);
        changed.id = ids[0].clone();
        let updated = update(f.engine(), changed.clone(), "k-u").await.unwrap();
        assert_eq!(updated.rule, changed);
        assert_eq!(
            listed_ids(f.engine()).await,
            [ids[0].as_str(), ids[1].as_str()]
        );
        assert_eq!(list(f.engine()).await[0].rule, changed);
        changed.id = RuleId::from_raw("ghost");
        let missing = update(f.engine(), changed, "k-ghost").await.unwrap_err();
        assert_eq!(missing, "not_found.rule");
        let announced = [ids[0].as_str(), ids[1].as_str(), ids[0].as_str()];
        assert_eq!(changed_rules(f.engine()).await, announced);
    })
    .await;
}

/// Désactiver puis réactiver se relit dans la liste et s'annonce à chaque fois
#[tokio::test]
async fn set_enabled_switches_the_rule() {
    within(async {
        let f = Fixture::start().await;
        let ids = create_named(f.engine(), &["a"]).await;
        let off = set_enabled(f.engine(), &ids[0], false, "k-off")
            .await
            .unwrap();
        assert!(!off.rule.enabled);
        assert!(!list(f.engine()).await[0].rule.enabled);
        let on = set_enabled(f.engine(), &ids[0], true, "k-on")
            .await
            .unwrap();
        assert!(on.rule.enabled);
        let ghost = set_enabled(f.engine(), &RuleId::from_raw("ghost"), true, "k-g").await;
        assert_eq!(ghost.unwrap_err(), "not_found.rule");
        assert_eq!(changed_rules(f.engine()).await.len(), 3);
    })
    .await;
}

/// Réordonner suit la liste donnée et n'annonce que les règles déplacées ; une liste incomplète est refusée sans effet
#[tokio::test]
async fn reorder_follows_the_given_order() {
    within(async {
        let f = Fixture::start().await;
        let ids = create_named(f.engine(), &["a", "b", "c"]).await;
        let order = vec![ids[1].clone(), ids[0].clone(), ids[2].clone()];
        assert_eq!(reorder(f.engine(), &order, "k-order").await.unwrap(), order);
        let names: Vec<&str> = order.iter().map(RuleId::as_str).collect();
        assert_eq!(listed_ids(f.engine()).await, names);
        let short = [ids[0].clone(), ids[1].clone()];
        let refused = reorder(f.engine(), &short, "k-short").await.unwrap_err();
        assert_eq!(refused, "invalid_input.parameter");
        assert_eq!(listed_ids(f.engine()).await, names);
        let moved = [ids[1].as_str(), ids[0].as_str()];
        assert_eq!(changed_rules(f.engine()).await[3..], moved);
    })
    .await;
}

/// Boîte visée disparue ou compte supprimé : la règle reste listée, invalide avec sa raison, et ne se réactive pas
#[tokio::test]
async fn rule_turns_invalid_when_its_world_changes() {
    within(async {
        let f = Fixture::start().await;
        let mover = create(f.engine(), move_rule(&f.account, "box-work"), "k-m").await;
        let mover = mover.unwrap().rule;
        let mine = subject_rule("Lettres", "Lettre", only(&f.account));
        let scoped = create(f.engine(), mine, "k-s").await.unwrap().rule;
        let gone = MailboxId::from_raw("box-work");
        f.db.write(|tx| tx.delete_mailbox(&gone)).unwrap();
        let listed = list(f.engine()).await;
        assert!(matches!(listed[0].status, RuleStatus::Invalid { .. }));
        assert_eq!(listed[1].status, RuleStatus::Valid);
        let enable = set_enabled(f.engine(), &mover.id, true, "k-on").await;
        assert_eq!(enable.unwrap_err(), "invalid_input.parameter");
        let delete = AccountRequest::DeleteAccount {
            account: f.account.clone(),
        };
        f.engine().execute(Request::Account(delete)).await.unwrap();
        let listed = list(f.engine()).await;
        assert_eq!(listed[1].rule.id, scoped.id);
        assert_eq!(listed[1].rule.scope, RuleScope::Accounts(Vec::new()));
        assert!(matches!(listed[1].status, RuleStatus::Invalid { .. }));
    })
    .await;
}
