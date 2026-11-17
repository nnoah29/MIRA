#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Outils de mémoire du chat : annonce selon le mode et l'activation, types de contenu, validation des propositions.

use mira_core::ai::tools::{is_proposal, kind_of, propose, specs_for, specs_with};
use mira_vocabulary::{
    ChatMode, ContentKind, MemoryKind, MemoryProposalAction, ToolCall, ToolState,
};

const NAMES: [&str; 3] = ["recall_memory", "remember", "forget_memory"];

/// Appel d'outil d'identifiant `c1`
fn call(name: &str, arguments: &str) -> ToolCall {
    ToolCall {
        id: "c1".into(),
        name: name.into(),
        arguments: arguments.into(),
    }
}

/// Les trois outils ne sont annoncés qu'à l'assistant, et seulement si la mémoire est activée
#[test]
fn memory_tools_are_offered_only_to_the_assistant_with_memory_enabled() {
    let names = |mode, memory| -> Vec<String> {
        specs_with(mode, memory)
            .into_iter()
            .map(|s| s.name)
            .collect()
    };
    for name in NAMES {
        assert!(names(ChatMode::Assistant, true).contains(&name.to_string()));
        assert!(!names(ChatMode::Assistant, false).contains(&name.to_string()));
        assert!(!names(ChatMode::Composer { selection: false }, true).contains(&name.to_string()));
    }
    assert_eq!(
        specs_for(ChatMode::Assistant),
        specs_with(ChatMode::Assistant, false)
    );
}

/// Le rappel est une lecture de type « mémoire » ; retenir et oublier sont des propositions sans type
#[test]
fn memory_tool_kinds_and_proposal_flags() {
    assert_eq!(kind_of("recall_memory"), Some(ContentKind::Memory));
    assert!(!is_proposal("recall_memory"));
    for name in ["remember", "forget_memory"] {
        assert!(is_proposal(name));
        assert_eq!(kind_of(name), None);
    }
}

/// Retenir valide le texte et rend la proposition, sans rien écrire
#[test]
fn remember_proposal_carries_text_and_kind() {
    let args = r#"{"text":"  Je signe « Noah »  ","kind":"preference"}"#;
    let ran = propose(ChatMode::Assistant, &call("remember", args));
    assert_eq!(ran.state, ToolState::Ok);
    let proposal = ran.memory.unwrap();
    assert_eq!(proposal.action, MemoryProposalAction::Remember);
    assert_eq!(
        (proposal.text.as_str(), proposal.kind),
        ("Je signe « Noah »", MemoryKind::Preference)
    );
    let default_kind = propose(ChatMode::Assistant, &call("remember", r#"{"text":"chat"}"#));
    assert_eq!(default_kind.memory.unwrap().kind, MemoryKind::Fact);
}

/// Un texte vide, trop long, sensible ou des arguments illisibles sont refusés, sans proposition
#[test]
fn remember_refuses_invalid_input() {
    let long = format!(r#"{{"text":"{}"}}"#, "a".repeat(1_001));
    let secret =
        r#"{"text":"ma clé sk-ant-api03-AbCdEfGhIjKlMnOpQrStUvWxYz0123456789"}"#.to_string();
    for arguments in [
        r#"{"text":"  "}"#.to_string(),
        long,
        secret,
        "pas du json".to_string(),
    ] {
        let ran = propose(ChatMode::Assistant, &call("remember", &arguments));
        assert_eq!(ran.state, ToolState::Error, "{arguments}");
        assert!(ran.memory.is_none());
    }
}

/// Oublier demande un identifiant et garde la raison ; sans identifiant : erreur
#[test]
fn forget_proposal_carries_the_id_and_requires_one() {
    let ok = propose(
        ChatMode::Assistant,
        &call("forget_memory", r#"{"id":"mem-1","reason":"obsolète"}"#),
    );
    let proposal = ok.memory.unwrap();
    assert_eq!(proposal.action, MemoryProposalAction::Forget);
    assert_eq!(
        (proposal.id.as_deref(), proposal.text.as_str()),
        (Some("mem-1"), "obsolète")
    );
    for arguments in [r#"{"id":"  "}"#, r#"{"reason":"x"}"#] {
        let refused = propose(ChatMode::Assistant, &call("forget_memory", arguments));
        assert_eq!(refused.state, ToolState::Error, "{arguments}");
    }
}
