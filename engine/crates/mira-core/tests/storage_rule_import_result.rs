#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Rapport d'import mémorisé avec la clé d'idempotence d'une règle.

mod support;

use mira_core::storage::repositories::rules::RuleIdempotencyEntry;
use mira_vocabulary::IdempotencyKey;
use support::accounts::database_with_account;

/// Le rapport mémorisé se relit ; sans rapport ou sans clé, rien n'est rendu
#[test]
fn stored_result_is_read_back_and_absent_otherwise() {
    let db = database_with_account("acc-1");
    let key = IdempotencyKey("k-1".to_string());
    let entry = RuleIdempotencyEntry {
        operation: "import_rules".to_string(),
        fingerprint: "f".to_string(),
        rule: None,
    };
    db.write(|tx| {
        assert_eq!(tx.rule_import_result(&key)?, None);
        tx.record_rule_idempotency(&key, &entry)?;
        assert_eq!(tx.rule_import_result(&key)?, None);
        tx.set_rule_import_result(&key, "{\"entries\":[]}")?;
        assert_eq!(
            tx.rule_import_result(&key)?.as_deref(),
            Some("{\"entries\":[]}")
        );
        Ok(())
    })
    .unwrap();
}
