#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Rattachement aux fils (D-15) : réponse, message connu, transfert sans en-tête, indice serveur.

mod support;
mod support_mirror;
mod support_reading;

use support_mirror::seeded::database_with_messages;
use support_reading::threading::{partition, store_and_assign, thread_of, threaded};

/// Une réponse qui cite la racine dans `References` rejoint le fil de la racine
#[test]
fn reply_joins_the_thread_of_its_root() {
    let db = database_with_messages(0);
    let root = threaded("root", "Plan", Some("<root>"), None, &[]);
    let reply = threaded(
        "reply",
        "Re: Plan",
        Some("<r1>"),
        Some("<root>"),
        &["<root>"],
    );
    let first = store_and_assign(&db, &root, 1_000, None);
    let second = store_and_assign(&db, &reply, 2_000, None);
    assert_eq!(first, second);
    assert_eq!(thread_of(&db, "reply"), Some(first));
}

/// Un message dont `In-Reply-To` désigne un message connu rejoint son fil, même si sa racine diffère
#[test]
fn in_reply_to_known_message_wins_over_a_different_root() {
    let db = database_with_messages(0);
    let root = threaded("root", "Plan", Some("<root>"), None, &[]);
    let side = threaded("side", "Autre", Some("<side>"), None, &[]);
    let odd = threaded(
        "odd",
        "Re: Plan",
        Some("<odd>"),
        Some("<root>"),
        &["<side>"],
    );
    let root_thread = store_and_assign(&db, &root, 1_000, None);
    let side_thread = store_and_assign(&db, &side, 1_500, None);
    let odd_thread = store_and_assign(&db, &odd, 2_000, None);
    assert_ne!(root_thread, side_thread);
    assert_eq!(odd_thread, root_thread);
}

/// Des transferts sans en-tête ni `Message-ID` se regroupent par objet normalisé
#[test]
fn forwards_without_headers_group_by_normalized_subject() {
    let db = database_with_messages(0);
    let one = threaded("one", "Fwd: Facture 12", None, None, &[]);
    let two = threaded("two", "TR: facture  12", None, None, &[]);
    let other = threaded("other", "Facture 13", None, None, &[]);
    for (message, at) in [(&one, 1_000), (&two, 2_000), (&other, 3_000)] {
        store_and_assign(&db, message, at, None);
    }
    assert_eq!(partition(&db, "acc-1"), [vec!["one", "two"], vec!["other"]]);
}

/// L'indice serveur regroupe des messages sans en-tête de réponse
#[test]
fn server_hint_groups_messages_without_reply_headers() {
    let db = database_with_messages(0);
    let one = threaded("one", "A", Some("<a>"), None, &[]);
    let two = threaded("two", "B", Some("<b>"), None, &[]);
    let three = threaded("three", "C", Some("<c>"), None, &[]);
    store_and_assign(&db, &one, 1_000, Some("42"));
    store_and_assign(&db, &two, 2_000, Some("42"));
    store_and_assign(&db, &three, 3_000, Some("43"));
    assert_eq!(partition(&db, "acc-1"), [vec!["one", "two"], vec!["three"]]);
}

/// Rattacher deux fois le même message donne le même fil
#[test]
fn assigning_twice_gives_the_same_thread() {
    let db = database_with_messages(0);
    let message = threaded("m", "Plan", Some("<m>"), None, &[]);
    let first = store_and_assign(&db, &message, 1_000, None);
    let again = db
        .write(|tx| {
            let other = mira_vocabulary::ThreadId::from_raw("t-autre");
            mira_core::threads::assign_thread(tx, &other, &message, None)
        })
        .unwrap();
    assert_eq!(first, again);
}

/// Deux comptes ne partagent jamais un fil, même avec le même `Message-ID`
#[test]
fn accounts_never_share_a_thread() {
    let db = database_with_messages(0);
    let mine = threaded("mine", "Plan", Some("<same>"), None, &[]);
    let mut theirs = threaded("theirs", "Plan", Some("<same>"), None, &[]);
    theirs.account = mira_vocabulary::AccountId::from_raw("acc-2");
    let left = store_and_assign(&db, &mine, 1_000, None);
    let right = store_and_assign(&db, &theirs, 2_000, None);
    assert_ne!(left, right);
}
