#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Dépôt des règles : ordre, portée, cascade de suppression de compte, file des messages à évaluer.

mod support;

use mira_vocabulary::{AccountId, MessageId, RuleId, RuleScope};
use support::accounts::{database_with_account, sample_account};
use support::rules::{id, rule_ids, sample_rule, scope_of};

/// Une règle insérée se relit à l'identique ; une règle inconnue donne rien
#[test]
fn insert_then_get_returns_same_rule() {
    let db = database_with_account("acc-1");
    let rule = sample_rule("r-1", scope_of(&["acc-1"]));
    db.write(|tx| tx.insert_rule(&rule)).unwrap();
    let loaded = db.read(|tx| tx.get_rule(&rule.id)).unwrap();
    assert_eq!(loaded, Some(rule));
    let unknown = db.read(|tx| tx.get_rule(&RuleId::from_raw("zz"))).unwrap();
    assert_eq!(unknown, None);
}

/// La liste suit `position` ; réordonner la change ; un doublon est refusé sans effet
#[test]
fn list_follows_position_and_reorder() {
    let db = database_with_account("acc-1");
    db.write(|tx| {
        for name in ["a", "b", "c"] {
            tx.insert_rule(&sample_rule(name, RuleScope::AllAccounts))?;
        }
        tx.reorder_rules(&[id("c"), id("a"), id("b")])
    })
    .unwrap();
    assert_eq!(rule_ids(&db), ["c", "a", "b"]);
    let duplicate = db.write(|tx| tx.reorder_rules(&[id("a"), id("a"), id("b")]));
    assert!(duplicate.is_err());
    assert_eq!(rule_ids(&db), ["c", "a", "b"]);
}

/// Une liste incomplète ou avec un identifiant inconnu est refusée, l'ordre reste inchangé
#[test]
fn reorder_with_missing_id_is_refused() {
    let db = database_with_account("acc-1");
    db.write(|tx| {
        tx.insert_rule(&sample_rule("a", RuleScope::AllAccounts))?;
        tx.insert_rule(&sample_rule("b", RuleScope::AllAccounts))
    })
    .unwrap();
    assert!(db.write(|tx| tx.reorder_rules(&[id("b")])).is_err());
    assert!(
        db.write(|tx| tx.reorder_rules(&[id("b"), id("zz")]))
            .is_err()
    );
    assert_eq!(rule_ids(&db), ["a", "b"]);
}

/// Supprimer un compte retire ses rattachements mais garde la règle, à portée vide
#[test]
fn deleting_account_removes_its_scope_rows_but_keeps_rule() {
    let db = database_with_account("acc-1");
    let rule = sample_rule("r-1", scope_of(&["acc-1"]));
    db.write(|tx| tx.insert_rule(&rule)).unwrap();
    db.write(|tx| tx.delete_account(&AccountId::from_raw("acc-1")))
        .unwrap();
    let loaded = db.read(|tx| tx.get_rule(&rule.id)).unwrap().unwrap();
    assert_eq!(loaded.scope, RuleScope::Accounts(vec![]));
    let rows = db.read(|tx| tx.table_row_count("rule_accounts")).unwrap();
    assert_eq!(rows, 0);
}

/// « Tous les comptes » ne crée aucune ligne de rattachement et se relit tel quel
#[test]
fn scope_all_roundtrips_without_account_rows() {
    let db = database_with_account("acc-1");
    db.write(|tx| tx.insert_account(&sample_account("acc-2")))
        .unwrap();
    let rule = sample_rule("r-1", RuleScope::AllAccounts);
    db.write(|tx| tx.insert_rule(&rule)).unwrap();
    let rows = db.read(|tx| tx.table_row_count("rule_accounts")).unwrap();
    assert_eq!(rows, 0);
    let loaded = db.read(|tx| tx.get_rule(&rule.id)).unwrap().unwrap();
    assert_eq!(loaded.scope, RuleScope::AllAccounts);
}

/// La file des messages à évaluer s'ajoute, se liste (sans doublon) et se vide
#[test]
fn pending_add_list_remove() {
    let db = database_with_account("acc-1");
    let message = MessageId::from_raw("m-1");
    db.write(|tx| {
        tx.add_pending(&message)?;
        tx.add_pending(&message)
    })
    .unwrap();
    let listed = db.read(|tx| tx.list_pending()).unwrap();
    assert_eq!(listed, std::slice::from_ref(&message));
    db.write(|tx| tx.remove_pending(&message)).unwrap();
    assert!(db.read(|tx| tx.list_pending()).unwrap().is_empty());
}
