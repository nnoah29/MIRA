//! Comparaison du miroir avec le serveur après une synchronisation : lignes, identifiants, drapeaux.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use mira_core::storage::Database;
use mira_testkit::imap::FakeMailStore;
use mira_vocabulary::MessageId;

use super::lookup::{MAILBOXES, USER, subject_of};
use crate::support_sync::mirror::{MirrorRow, ids_by_subject, mirror_rows, stored_messages};

/// Lignes que le miroir doit montrer : une par copie présente sur le serveur
fn server_rows(store: &FakeMailStore) -> Vec<MirrorRow> {
    let mut rows = Vec::new();
    for name in MAILBOXES {
        for view in store.messages(USER, name) {
            let key = u8::try_from(view.gm_msgid - 100).unwrap();
            let has = |flag: &str| view.flags.iter().any(|found| found == flag);
            rows.push(MirrorRow {
                mailbox: name.to_string(),
                uid: view.uid,
                subject: subject_of(key),
                seen: has("\\Seen"),
                flagged: has("\\Flagged"),
            });
        }
    }
    rows.sort();
    rows
}

/// Invariants 8 et 12 : le miroir égale le serveur, sans message sans apparition, drapeaux identiques dans toutes les boîtes
fn check_rows(db: &Database, store: &FakeMailStore) {
    let rows = mirror_rows(db);
    assert_eq!(rows, server_rows(store), "le serveur gagne");
    let subjects: BTreeSet<&str> = rows.iter().map(|row| row.subject.as_str()).collect();
    assert_eq!(
        stored_messages(db),
        subjects.len(),
        "un message = une ligne, sans orphelin"
    );
    let mut flags: BTreeMap<&str, (bool, bool)> = BTreeMap::new();
    for row in &rows {
        let seen_before = flags.insert(&row.subject, (row.seen, row.flagged));
        assert!(seen_before.is_none_or(|old| old == (row.seen, row.flagged)));
    }
}

/// Invariant 7 : un message présent à deux synchronisations successives garde son identifiant interne
pub fn check_after_sync(
    db: &Database,
    store: &FakeMailStore,
    known: &mut HashMap<String, MessageId>,
) {
    check_rows(db, store);
    let current = ids_by_subject(db);
    for (subject, id) in &current {
        if let Some(before) = known.get(subject) {
            assert_eq!(before, id, "identifiant interne de {subject}");
        }
    }
    *known = current;
}
