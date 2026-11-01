#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Outil `propose_mailbox_action` : validation, mode assistant seulement, catalogue.

mod ai_tools_draft_support;
mod ai_tools_support;
mod support;
mod support_api;
mod support_mirror;
mod support_read;

use ai_tools_draft_support::{action, names};
use ai_tools_support::call;
use mira_core::ai::tools::{is_proposal, kind_of, propose, summary_of};
use mira_vocabulary::{ChatMode, MailActionKind, ToolState};
use serde_json::json;

/// Une demande valide donne une proposition, sans doublon d'identifiant, et dit au modèle que l'utilisateur confirmera
#[test]
fn valid_action_gives_a_proposal() {
    let run = action(json!({
        "action": "archive",
        "messages": ["m1", "m2", "m1"],
        "reason": "Newsletters déjà lues"
    }));
    assert_eq!(run.state, ToolState::Ok, "{}", run.text);
    let proposal = run.action.unwrap();
    assert_eq!(proposal.action, MailActionKind::Archive);
    assert_eq!(proposal.messages, ["m1", "m2"]);
    assert_eq!(proposal.reason.as_deref(), Some("Newsletters déjà lues"));
    assert!(run.text.contains("confirm"), "{}", run.text);
    assert!(run.proposal.is_none());
}

/// Les identifiants sont obligatoires, non blancs, de 200 caractères au plus et de 100 au plus
#[test]
fn ids_are_required_and_bounded() {
    let many = |n: usize| -> Vec<String> { (0..n).map(|i| format!("m{i}")).collect() };
    assert_eq!(
        action(json!({"action": "trash", "messages": many(100)})).state,
        ToolState::Ok
    );
    for messages in [
        json!([]),
        json!(["  "]),
        json!(many(101)),
        json!(["a".repeat(201)]),
    ] {
        let run = action(json!({"action": "trash", "messages": messages}));
        assert_eq!(run.state, ToolState::Error);
        assert!(run.action.is_none());
    }
}

/// Une action inconnue ou absente et des arguments illisibles sont refusés
#[test]
fn unknown_actions_are_refused() {
    for arguments in [
        json!({"action": "delete_forever", "messages": ["m1"]}),
        json!({"messages": ["m1"]}),
        json!({"action": "archive"}),
    ] {
        let run = action(arguments);
        assert_eq!(run.state, ToolState::Error);
        assert!(run.action.is_none());
    }
}

/// La raison tient sur une ligne de 200 caractères au plus
#[test]
fn reason_is_one_short_line() {
    let ok = action(json!({"action": "star", "messages": ["m1"], "reason": "a".repeat(200)}));
    assert_eq!(ok.state, ToolState::Ok);
    for reason in ["a".repeat(201), "ligne 1\nligne 2".to_string()] {
        let run = action(json!({"action": "star", "messages": ["m1"], "reason": reason}));
        assert_eq!(run.state, ToolState::Error);
    }
}

/// Seul le mode assistant l'annonce ; hors de ce mode l'appel est indisponible
#[test]
fn only_the_assistant_mode_offers_it() {
    assert!(names(ChatMode::Assistant).contains(&"propose_mailbox_action".to_string()));
    for selection in [false, true] {
        let mode = ChatMode::Composer { selection };
        assert!(!names(mode).contains(&"propose_mailbox_action".to_string()));
        let run = propose(
            mode,
            &call(
                "propose_mailbox_action",
                json!({"action": "archive", "messages": ["m1"]}),
            ),
        );
        assert_eq!(run.state, ToolState::Error);
        assert!(run.action.is_none());
    }
}

/// Le catalogue la connaît comme proposition, sans type de contenu, et l'extrait d'appel est la raison
#[test]
fn catalog_knows_it() {
    assert!(is_proposal("propose_mailbox_action"));
    assert_eq!(kind_of("propose_mailbox_action"), None);
    let summary = summary_of(&call(
        "propose_mailbox_action",
        json!({"reason": "Spam évident", "action": "trash", "messages": ["m1"]}),
    ));
    assert_eq!(summary, "Spam évident");
}
