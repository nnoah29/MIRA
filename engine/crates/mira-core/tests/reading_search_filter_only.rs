#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Recherche sans texte : les filtres seuls listent les messages, l'absence de texte et de filtre ne rend rien.

mod support;
mod support_mirror;
mod support_reading;

use mira_core::reading::{MessageFilters, SearchFilters};
use mira_vocabulary::Timestamp;
use support_reading::indexed::database_with_indexed_messages;
use support_reading::queries::found;

/// Filtres de recherche dont seul le champ modifié par `edit` est posé
fn only(edit: fn(&mut MessageFilters)) -> SearchFilters {
    let mut common = MessageFilters::default();
    edit(&mut common);
    SearchFilters {
        common,
        ..SearchFilters::default()
    }
}

/// Sans texte, un filtre actif suffit à lister les messages concernés
#[test]
fn a_filter_alone_lists_matching_messages() {
    let db = database_with_indexed_messages();
    assert_eq!(found(&db, "", &only(|f| f.has_attachments = true)), ["s1"]);
    assert_eq!(found(&db, "  ", &only(|f| f.flagged_only = true)), ["s2"]);
    let since = only(|f| f.since = Some(Timestamp::from_millis(2_000)));
    assert!(found(&db, "", &since).contains(&"s3".to_string()));
}

/// Sans texte ni filtre, rien n'est rendu : on ne liste pas tout l'index
#[test]
fn no_text_and_no_filter_returns_nothing() {
    let db = database_with_indexed_messages();
    assert!(found(&db, "", &SearchFilters::default()).is_empty());
}
