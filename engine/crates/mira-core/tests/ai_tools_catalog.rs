#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Catalogue d'outils : noms, schémas, types de contenu, extrait d'appel et ligne de message.

mod ai_tools_support;
mod support;
mod support_api;
mod support_mirror;
mod support_read;

use ai_tools_support::call;
use mira_core::ai::tools::render::mail_line;
use mira_core::ai::tools::{kind_of, specs, summary_of};
use mira_vocabulary::ContentKind;
use serde_json::{Value, json};
use support_mirror::fixtures::sample_message;

/// Les outils de recherche et de liste sont annoncés avec leur description
#[test]
fn specs_list_search_and_list_tools() {
    let names: Vec<String> = specs().into_iter().map(|spec| spec.name).collect();
    assert!(names.contains(&"search_messages".to_string()));
    assert!(names.contains(&"list_messages".to_string()));
    assert!(specs().iter().all(|spec| !spec.description.is_empty()));
}

/// Chaque schéma d'arguments est un objet JSON valide de type object
#[test]
fn every_spec_schema_is_a_json_object() {
    for spec in specs() {
        let schema: Value = serde_json::from_str(&spec.parameters_schema).unwrap();
        assert_eq!(schema["type"], "object", "{}", spec.name);
    }
}

/// Un outil inconnu n'a pas de type de contenu
#[test]
fn unknown_tool_has_no_kind() {
    assert_eq!(kind_of("delete_everything"), None);
}

/// Les listes de messages sont du type `mail_list`
#[test]
fn search_and_list_return_mail_lists() {
    assert_eq!(kind_of("search_messages"), Some(ContentKind::MailList));
    assert_eq!(kind_of("list_messages"), Some(ContentKind::MailList));
}

/// L'extrait d'un appel de recherche est sa requête, coupée à 80 caractères sur une seule ligne
#[test]
fn summary_is_the_query_cut_at_80_chars() {
    let long = format!("a\nb{}", "x".repeat(200));
    let summary = summary_of(&call("search_messages", json!({"query": long})));
    assert_eq!(summary.chars().count(), 80);
    assert!(!summary.contains('\n'));
    assert_eq!(
        summary_of(&call("search_messages", json!({"query": "devis"}))),
        "devis"
    );
}

/// La ligne d'un message porte son identifiant et un aperçu coupé à 200 caractères
#[test]
fn mail_line_cuts_preview_at_200_chars() {
    let mut message = sample_message("m1", "acc-1");
    message.preview = "é".repeat(300);
    let line = mail_line(&message);
    assert!(line.starts_with("id=m1 | de: Bob <bob@example.org> | "));
    assert!(line.contains("Sujet m1"));
    assert!(line.ends_with(&"é".repeat(200)));
}
