#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Domaine « rule » par la façade : une règle appliquée aux messages déjà présents agit comme à l'arrivée.

mod support;
mod support_api;
mod support_imap;
mod support_mirror;
mod support_rule_api;
mod support_write;

use mira_core::api::{Response, RuleRequest, RuleResponse};
use mira_vocabulary::{MessageId, RuleId, RuleScope};
use support_imap::within;
use support_rule_api::calls::{create, key, rule_call};
use support_rule_api::samples::subject_rule;
use support_rule_api::seed::seed_lots;
use support_write::fixture::Fixture;

/// Demande l'application de la règle à l'existant et rend le bilan
async fn apply(f: &Fixture, id: &RuleId, key_text: &str) -> Result<(u64, u64), String> {
    let request = RuleRequest::ApplyRuleToExisting {
        rule: id.clone(),
        idempotency_key: key(key_text),
    };
    match rule_call(f.engine(), request).await {
        Ok(Response::Rule(RuleResponse::Preview(report))) => Ok((report.matched, report.scanned)),
        Ok(other) => panic!("réponse inattendue : {other:?}"),
        Err(error) => Err(error.code),
    }
}

/// Nombre d'actions du compte, tous états confondus
fn queue_len(f: &Fixture) -> usize {
    f.db.read(|tx| tx.list_actions(&f.account, &[]))
        .unwrap()
        .len()
}

/// Lu ou non, tel que le miroir local le montre
fn is_read(f: &Fixture, id: &str) -> bool {
    f.db.read(|tx| tx.get_message(&MessageId::from_raw(id)))
        .unwrap()
        .unwrap()
        .flags
        .seen
}

/// Les messages présents que la règle touche reçoivent ses actions, les autres et la Corbeille restent intacts
#[tokio::test]
async fn existing_messages_receive_the_actions_of_the_rule() {
    within(async {
        let f = Fixture::start().await;
        seed_lots(&f.db, &f.account, 4);
        let rule = subject_rule("Lots", "Lot", RuleScope::AllAccounts);
        let view = create(f.engine(), rule, "create-1").await.unwrap();
        let (matched, scanned) = apply(&f, &view.rule.id, "apply-1").await.unwrap();
        assert_eq!((matched, scanned), (4, 7));
        assert!(is_read(&f, "lot-2"));
        assert!(!is_read(&f, "lot-4"));
    })
    .await;
}

/// Rejouer la demande sous la même clé ne met rien de plus en file
#[tokio::test]
async fn applying_twice_under_the_same_key_queues_nothing_more() {
    within(async {
        let f = Fixture::start().await;
        seed_lots(&f.db, &f.account, 3);
        let rule = subject_rule("Lots", "Lot", RuleScope::AllAccounts);
        let view = create(f.engine(), rule, "create-1").await.unwrap();
        let id = view.rule.id;
        apply(&f, &id, "apply-1").await.unwrap();
        let queued = queue_len(&f);
        apply(&f, &id, "apply-1").await.unwrap();
        let again = queue_len(&f);
        assert_eq!(queued, again);
    })
    .await;
}

/// Une règle inconnue est refusée
#[tokio::test]
async fn applying_an_unknown_rule_is_refused() {
    within(async {
        let f = Fixture::start().await;
        let code = apply(&f, &RuleId::from_raw("nope"), "apply-1").await;
        assert_eq!(code, Err("not_found.rule".to_string()));
    })
    .await;
}
