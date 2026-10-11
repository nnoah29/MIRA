#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Recherche locale filtrée : plein texte, expéditeur, destinataire, période, pièce jointe, boîte, drapeaux.

mod support;
mod support_mirror;
mod support_reading;

use mira_core::reading::{MessageFilters, SearchFilters, search};
use mira_vocabulary::{AccountId, MailboxId, PageRequest, Timestamp};
use support_reading::indexed::database_with_indexed_messages;
use support_reading::queries::{found, profile};
use support_reading::views::page_ids;

/// Le texte est cherché dans le sujet, l'expéditeur et le corps (sans accents), du plus récent au plus ancien
#[test]
fn text_search_spans_accounts_newest_first() {
    let db = database_with_indexed_messages();
    let none = SearchFilters::default();
    assert_eq!(found(&db, "facture", &none), ["s3", "s2", "s1"]);
    assert_eq!(found(&db, "electricite", &none), ["s2"]);
    assert_eq!(found(&db, "bob", &none), ["s3", "s1"]);
    assert!(found(&db, "   ", &none).is_empty());
    let one = [AccountId::from_raw("acc-1")];
    let page = search(&db, &one, "facture", &none, &PageRequest::default()).unwrap();
    assert_eq!(page_ids(&page), ["s2", "s1"]);
}

/// Les filtres expéditeur et destinataire (à ou copie) restreignent les résultats
#[test]
fn sender_and_recipient_filters_narrow_results() {
    let db = database_with_indexed_messages();
    let from = |text: &str| SearchFilters {
        common: MessageFilters {
            from_contains: Some(text.to_string()),
            ..MessageFilters::default()
        },
        ..SearchFilters::default()
    };
    assert_eq!(found(&db, "facture", &from("bob")), ["s3", "s1"]);
    assert_eq!(found(&db, "facture", &from("example.net")), ["s2"]);
    let to = SearchFilters {
        to_contains: Some("DAVE@".to_string()),
        ..SearchFilters::default()
    };
    assert_eq!(found(&db, "facture", &to), ["s2"]);
    let alice = SearchFilters {
        to_contains: Some("alice".to_string()),
        ..SearchFilters::default()
    };
    assert_eq!(found(&db, "facture", &alice), ["s3", "s2", "s1"]);
}

/// Période, pièce jointe et drapeaux se combinent avec le texte
#[test]
fn period_attachment_and_flag_filters_combine() {
    let db = database_with_indexed_messages();
    let filters = |edit: fn(&mut MessageFilters)| {
        let mut common = MessageFilters::default();
        edit(&mut common);
        SearchFilters {
            common,
            ..SearchFilters::default()
        }
    };
    let period = filters(|f| f.since = Some(Timestamp::from_millis(2_000)));
    assert_eq!(found(&db, "facture", &period), ["s3", "s2"]);
    assert_eq!(
        found(&db, "facture", &filters(|f| f.has_attachments = true)),
        ["s1"]
    );
    assert_eq!(
        found(&db, "facture", &filters(|f| f.flagged_only = true)),
        ["s2"]
    );
    assert_eq!(
        found(&db, "facture", &filters(|f| f.unread_only = true)),
        ["s3", "s2"]
    );
}

/// Le filtre de boîte ne garde que les messages apparaissant dans cette boîte
#[test]
fn mailbox_filter_keeps_only_that_mailbox() {
    let db = database_with_indexed_messages();
    let in_box = |id: &str| SearchFilters {
        mailbox: Some(MailboxId::from_raw(id)),
        ..SearchFilters::default()
    };
    assert_eq!(found(&db, "facture", &in_box("label-1")), ["s2"]);
    assert_eq!(found(&db, "facture", &in_box("box-1")), ["s1"]);
    assert!(found(&db, "facture", &in_box("sent-1")).is_empty());
}

/// Les résultats se parcourent page par page, sans doublon
#[test]
fn search_results_paginate_with_a_cursor() {
    let db = database_with_indexed_messages();
    let none = SearchFilters::default();
    let request = PageRequest {
        cursor: None,
        limit: 2,
    };
    let first = search(&db, &profile(), "facture", &none, &request).unwrap();
    assert_eq!(page_ids(&first), ["s3", "s2"]);
    let next = PageRequest {
        cursor: first.next_cursor,
        limit: 2,
    };
    let second = search(&db, &profile(), "facture", &none, &next).unwrap();
    assert_eq!(page_ids(&second), ["s1"]);
    assert!(second.next_cursor.is_none());
}

/// La syntaxe de recherche du moteur n'est jamais interprétée : aucune injection, aucune erreur
#[test]
fn search_syntax_is_never_interpreted() {
    let db = database_with_indexed_messages();
    let none = SearchFilters::default();
    for text in [
        "facture OR plage",
        "\"",
        "NEAR(a b)",
        "subject:facture",
        "*",
        "facture -plage",
    ] {
        assert!(found(&db, text, &none).is_empty(), "requête {text:?}");
    }
}
