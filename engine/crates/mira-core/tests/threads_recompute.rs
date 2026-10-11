#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Recalcul des fils (invariant 13) : sans perte, mêmes regroupements, identifiants conservés.

mod support;
mod support_mirror;
mod support_reading;

use mira_core::ports::SequentialIds;
use mira_core::storage::Database;
use mira_core::threads::recompute_threads;
use mira_vocabulary::AccountId;
use support_mirror::seeded::database_with_messages;
use support_reading::threading::{partition, store_and_assign, thread_of, threaded};

/// Base avec une conversation, un fil par indice serveur, un transfert sans en-tête et un message isolé
fn mixed_conversations() -> Database {
    let db = database_with_messages(0);
    let messages = [
        threaded("root", "Plan", Some("<root>"), None, &[]),
        threaded("a", "Re: Plan", Some("<a>"), Some("<root>"), &["<root>"]),
        threaded(
            "b",
            "Re: Re: Plan",
            Some("<b>"),
            Some("<a>"),
            &["<root>", "<a>"],
        ),
        threaded("h1", "H", Some("<h1>"), None, &[]),
        threaded("h2", "H bis", Some("<h2>"), None, &[]),
        threaded("fwd", "Fwd: Plan", None, None, &[]),
        threaded("alone", "Seul", Some("<alone>"), None, &[]),
    ];
    for (index, message) in messages.iter().enumerate() {
        let hint = message.id.as_str().starts_with('h').then_some("srv-1");
        store_and_assign(&db, message, 1_000 * (index as i64 + 1), hint);
    }
    db
}

/// Scénario invariant 13 : recalculer les fils redonne exactement les mêmes regroupements
#[test]
fn invariant_13_threads_recomputed_identically() {
    let db = mixed_conversations();
    let before = partition(&db, "acc-1");
    assert_eq!(before.len(), 4);
    let ids = SequentialIds::new();
    let count = recompute_threads(&db, &ids, &AccountId::from_raw("acc-1")).unwrap();
    assert_eq!(count, 7);
    assert_eq!(partition(&db, "acc-1"), before);
}

/// Les identifiants de fil survivent au recalcul quand la clé de racine ne change pas
#[test]
fn recompute_keeps_thread_ids() {
    let db = mixed_conversations();
    let root_before = thread_of(&db, "root").unwrap();
    let hint_before = thread_of(&db, "h1").unwrap();
    let ids = SequentialIds::new();
    recompute_threads(&db, &ids, &AccountId::from_raw("acc-1")).unwrap();
    assert_eq!(thread_of(&db, "b"), Some(root_before));
    assert_eq!(thread_of(&db, "h2"), Some(hint_before));
}

/// Recalculer deux fois de suite est idempotent
#[test]
fn recompute_is_idempotent() {
    let db = mixed_conversations();
    let ids = SequentialIds::new();
    let account = AccountId::from_raw("acc-1");
    recompute_threads(&db, &ids, &account).unwrap();
    let once = partition(&db, "acc-1");
    recompute_threads(&db, &ids, &account).unwrap();
    assert_eq!(partition(&db, "acc-1"), once);
}

/// Le recalcul d'un compte ne touche pas aux fils des autres comptes
#[test]
fn recompute_leaves_other_accounts_alone() {
    let db = mixed_conversations();
    let mut theirs = threaded("theirs", "Plan", Some("<t>"), None, &[]);
    theirs.account = AccountId::from_raw("acc-2");
    let thread = store_and_assign(&db, &theirs, 9_000, None);
    let ids = SequentialIds::new();
    recompute_threads(&db, &ids, &AccountId::from_raw("acc-1")).unwrap();
    assert_eq!(thread_of(&db, "theirs"), Some(thread));
}

/// Un compte sans message se recalcule sans erreur
#[test]
fn recompute_of_empty_account_does_nothing() {
    let db = database_with_messages(0);
    let ids = SequentialIds::new();
    let count = recompute_threads(&db, &ids, &AccountId::from_raw("acc-1")).unwrap();
    assert_eq!(count, 0);
}
