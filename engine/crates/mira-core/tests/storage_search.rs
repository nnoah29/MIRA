#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Recherche FTS5 : accents, requête échappée, portée par compte, reconstruction.

mod support;
mod support_mirror;

use mira_vocabulary::{MessageId, PageRequest};
use support_mirror::search::{found, index};
use support_mirror::seeded::database_with_messages;

/// Les accents sont ignorés dans les deux sens : `été` trouve `ete` et inversement
#[test]
fn accents_are_ignored_both_ways() {
    let db = database_with_messages(2);
    index(&db, "msg-1", "Plan de l'été", "rendez-vous");
    index(&db, "msg-2", "Autre", "pas de rapport avec ete");
    assert_eq!(found(&db, None, "ete"), ["msg-2", "msg-1"]);
    assert_eq!(found(&db, None, "été"), ["msg-2", "msg-1"]);
    assert!(found(&db, None, "hiver").is_empty());
}

/// Une requête piégée n'injecte aucune syntaxe FTS et ne casse rien
#[test]
fn hostile_queries_do_not_inject_fts_syntax() {
    let db = database_with_messages(1);
    index(&db, "msg-1", "alpha beta", "corps");
    for query in [
        "\"a\" OR 1",
        "alpha OR zzz",
        "a*",
        "***",
        "\"",
        "NEAR(a b)",
        "subject:alpha",
        "   ",
    ] {
        let result = db.read(|tx| tx.search_messages(None, query, &PageRequest::default()));
        assert!(result.is_ok(), "requête refusée : {query}");
    }
    assert!(found(&db, None, "alpha OR zzz").is_empty());
    assert_eq!(found(&db, None, "alpha beta"), ["msg-1"]);
    assert!(found(&db, None, "subject:alpha").is_empty());
}

/// La recherche respecte le compte demandé et la désindexation retire le message
#[test]
fn search_is_scoped_by_account_and_unindex_removes() {
    let db = database_with_messages(1);
    index(&db, "msg-1", "facture", "montant");
    assert_eq!(found(&db, Some("acc-1"), "facture"), ["msg-1"]);
    assert!(found(&db, Some("acc-2"), "facture").is_empty());
    db.write(|tx| tx.unindex_message(&MessageId::from_raw("msg-1")))
        .unwrap();
    assert!(found(&db, None, "facture").is_empty());
}

/// La reconstruction réindexe sujet, adresses et corps (cache d'analyse, aperçu à défaut)
#[test]
fn rebuild_reindexes_subject_addresses_and_body() {
    let db = database_with_messages(2);
    db.write(|tx| tx.store_parsed(&MessageId::from_raw("msg-1"), "contenu secretissime", None))
        .unwrap();
    db.write(|tx| tx.rebuild_search_index()).unwrap();
    assert_eq!(found(&db, None, "secretissime"), ["msg-1"]);
    assert_eq!(found(&db, None, "Aperçu msg-2"), ["msg-2"]);
    assert_eq!(found(&db, None, "bob@example.org").len(), 2);
    assert_eq!(found(&db, None, "alice"), ["msg-2", "msg-1"]);
}

/// Les résultats sont paginés par curseur, sans doublon ni trou
#[test]
fn search_results_are_cursor_paginated() {
    let db = database_with_messages(5);
    for index_number in 1..=5 {
        index(&db, &format!("msg-{index_number}"), "commun", "corps");
    }
    let mut request = PageRequest {
        cursor: None,
        limit: 2,
    };
    let mut seen = Vec::new();
    loop {
        let page = db
            .read(|tx| tx.search_messages(None, "commun", &request))
            .unwrap();
        seen.extend(page.items.iter().map(|m| m.id.as_str().to_string()));
        match page.next_cursor {
            Some(cursor) => request.cursor = Some(cursor),
            None => break,
        }
    }
    assert_eq!(seen, ["msg-5", "msg-4", "msg-3", "msg-2", "msg-1"]);
}
