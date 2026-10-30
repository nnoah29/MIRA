#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Journal des envois d'IA, outils : noms et nombres d'appels, types envoyés ajoutés, caractères cumulés.

mod support_ai_journal;

use mira_core::storage::Database;
use mira_vocabulary::ContentKind;
use serde_json::Value;
use support_ai_journal::tools::{count, finished_with};
use support_ai_journal::{entry, finish, insert_all};

/// Les outils appelés sont gardés avec leur nombre d'appels
#[test]
fn finish_stores_tool_counts() {
    let db = Database::open_in_memory().unwrap();
    insert_all(&db, &[("j-1", "op-1", 10)]);
    let mut done = finish("end_turn", 0);
    done.tools = vec![count("search_messages", 2), count("get_thread", 1)];
    let row = finished_with(&db, done);
    assert_eq!(
        row.tools,
        [count("search_messages", 2), count("get_thread", 1)]
    );
}

/// Les types envoyés en cours de route s'ajoutent à ceux de l'ouverture, sans doublon
#[test]
fn finish_merges_extra_kinds_without_duplicates() {
    let db = Database::open_in_memory().unwrap();
    insert_all(&db, &[("j-1", "op-1", 10)]);
    let mut done = finish("end_turn", 0);
    done.extra_kinds = vec![ContentKind::MailList, ContentKind::MailMessage];
    let row = finished_with(&db, done);
    assert_eq!(row.kinds, [ContentKind::MailMessage, ContentKind::MailList]);
}

/// Les caractères envoyés au fil des tours s'ajoutent à ceux de l'ouverture
#[test]
fn finish_adds_extra_chars() {
    let db = Database::open_in_memory().unwrap();
    insert_all(&db, &[("j-1", "op-1", 10)]);
    let before = entry("j-1", "op-1", 10).chars;
    let mut done = finish("end_turn", 0);
    done.extra_chars = 100;
    assert_eq!(finished_with(&db, done).chars, before + 100);
}

/// Une ligne ouverte, ou complétée sans outil, n'en a aucun
#[test]
fn rows_without_tools_have_none() {
    let db = Database::open_in_memory().unwrap();
    insert_all(&db, &[("j-1", "op-1", 10)]);
    assert!(finished_with(&db, finish("end_turn", 0)).tools.is_empty());
}

/// Une ligne sérialisée ne porte, pour les outils, que des noms et des nombres
#[test]
fn journal_row_json_has_only_names_and_counts() {
    let db = Database::open_in_memory().unwrap();
    insert_all(&db, &[("j-1", "op-1", 10)]);
    let mut done = finish("end_turn", 0);
    done.tools = vec![count("search_messages", 2)];
    let row = serde_json::to_value(finished_with(&db, done)).unwrap();
    let keys: Vec<&String> = row["tools"][0].as_object().unwrap().keys().collect();
    assert_eq!(keys, ["count", "name"]);
    assert_eq!(row["tools"][0]["count"], Value::from(2));
}
