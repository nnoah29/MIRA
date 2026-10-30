#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Catalogue d'outils : noms, schémas, types de contenu, extrait d'appel et ligne de message.

mod ai_tools_support;
mod support;
mod support_api;
mod support_mirror;
mod support_read;

use ai_tools_support::call;
use mira_core::ai::tools::{kind_of, specs, summary_of};
use mira_vocabulary::ContentKind;
use serde_json::{Value, json};

/// Les outils de recherche et de liste sont annoncés avec leur description
#[test]
fn specs_list_search_and_list_tools() {
    let names: Vec<String> = specs().into_iter().map(|spec| spec.name).collect();
    assert!(names.contains(&"search_messages".to_string()));
    assert!(names.contains(&"list_messages".to_string()));
    for name in ["get_message", "get_thread", "list_mailboxes"] {
        assert!(names.contains(&name.to_string()), "{name}");
    }
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

/// Lire un message ou un fil donne `mail_message`, lister les boîtes `tool_result`
#[test]
fn read_tools_have_their_kinds() {
    assert_eq!(kind_of("get_message"), Some(ContentKind::MailMessage));
    assert_eq!(kind_of("get_thread"), Some(ContentKind::MailMessage));
    assert_eq!(kind_of("list_mailboxes"), Some(ContentKind::ToolResult));
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
