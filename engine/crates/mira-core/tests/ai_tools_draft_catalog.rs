#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Outils de brouillon : jeux d'outils par mode, schémas, types de contenu et extrait d'appel.

mod ai_tools_draft_support;
mod ai_tools_support;
mod support;
mod support_api;
mod support_mirror;
mod support_read;

use ai_tools_draft_support::names;
use ai_tools_support::call;
use mira_core::ai::tools::{is_proposal, kind_of, specs, specs_for, summary_of};
use mira_vocabulary::{ChatMode, ContentKind};
use serde_json::{Value, json};

/// L'assistant peut composer un message et lire les identités, sans pouvoir éditer un brouillon ouvert
#[test]
fn assistant_mode_offers_compose_not_edit() {
    let found = names(ChatMode::Assistant);
    for name in [
        "compose_draft",
        "list_identities",
        "search_messages",
        "get_message",
    ] {
        assert!(found.contains(&name.to_string()), "{name}");
    }
    assert!(!found.contains(&"edit_draft".to_string()));
}

/// Le composeur peut éditer le brouillon ouvert, sans pouvoir en composer un autre
#[test]
fn composer_modes_offer_edit_not_compose() {
    for selection in [false, true] {
        let found = names(ChatMode::Composer { selection });
        assert!(found.contains(&"edit_draft".to_string()), "{selection}");
        assert!(
            found.contains(&"list_identities".to_string()),
            "{selection}"
        );
        assert!(!found.contains(&"compose_draft".to_string()), "{selection}");
    }
}

/// Chaque schéma d'arguments est un objet JSON, dans tous les modes
#[test]
fn every_draft_schema_is_a_json_object() {
    let modes = [
        ChatMode::Assistant,
        ChatMode::Composer { selection: false },
        ChatMode::Composer { selection: true },
    ];
    for mode in modes {
        for spec in specs_for(mode) {
            let schema: Value = serde_json::from_str(&spec.parameters_schema).unwrap();
            assert_eq!(schema["type"], "object", "{}", spec.name);
            assert!(!spec.description.is_empty(), "{}", spec.name);
        }
    }
}

/// Les outils de brouillon n'ont pas de type de contenu : ils n'envoient rien au service
#[test]
fn proposal_tools_have_no_content_kind() {
    for name in ["compose_draft", "edit_draft"] {
        assert!(is_proposal(name), "{name}");
        assert_eq!(kind_of(name), None, "{name}");
    }
    assert!(!is_proposal("search_messages"));
    assert!(!is_proposal("list_identities"));
    assert_eq!(kind_of("list_identities"), Some(ContentKind::ToolResult));
}

/// L'extrait d'un appel de brouillon est son objet sur une ligne, coupé à 80 caractères
#[test]
fn summary_of_a_draft_call_is_its_subject() {
    let long = format!("a\nb{}", "x".repeat(200));
    let summary = summary_of(&call(
        "compose_draft",
        json!({"subject": long, "body": "x"}),
    ));
    assert_eq!(summary.chars().count(), 80);
    assert!(!summary.contains('\n'));
    assert_eq!(
        summary_of(&call("edit_draft", json!({"subject": "Fin de stage"}))),
        "Fin de stage"
    );
    assert_eq!(summary_of(&call("edit_draft", json!({"body": "x"}))), "");
}

/// `specs()` garde les seuls outils de lecture : lire n'a pas changé, les identités s'y ajoutent
#[test]
fn specs_lists_read_tools_only() {
    let found: Vec<String> = specs().into_iter().map(|spec| spec.name).collect();
    assert!(found.contains(&"list_identities".to_string()));
    assert!(!found.contains(&"compose_draft".to_string()));
    assert!(!found.contains(&"edit_draft".to_string()));
}
