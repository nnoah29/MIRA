#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Outil `edit_draft` : modification du brouillon ouvert ou d'un passage sélectionné, outils hors mode.

mod ai_tools_draft_support;
mod ai_tools_support;
mod support;
mod support_api;
mod support_mirror;
mod support_read;

use ai_tools_draft_support::edit;
use ai_tools_support::call;
use mira_core::ai::tools::propose;
use mira_vocabulary::{ChatMode, DraftProposalKind, ToolState};
use serde_json::json;

const WHOLE: ChatMode = ChatMode::Composer { selection: false };
const SELECTION: ChatMode = ChatMode::Composer { selection: true };

/// Seuls les champs donnés changent, dans une proposition « édition »
#[test]
fn edit_changes_only_given_fields() {
    let run = edit(WHOLE, json!({"subject": "Nouveau", "body": "Texte"}));
    assert_eq!(run.state, ToolState::Ok);
    let proposal = run.proposal.unwrap();
    assert_eq!(proposal.kind, DraftProposalKind::Edit);
    assert_eq!(proposal.subject.as_deref(), Some("Nouveau"));
    assert_eq!(proposal.body.as_deref(), Some("Texte"));
    assert!(proposal.to.is_none() && proposal.cc.is_none() && proposal.selection.is_none());
}

/// Un appel sans aucun champ est refusé
#[test]
fn edit_needs_at_least_one_field() {
    for mode in [WHOLE, SELECTION] {
        let run = edit(mode, json!({}));
        assert_eq!(run.state, ToolState::Error);
        assert!(run.proposal.is_none());
    }
}

/// Avec une sélection, seul le texte de remplacement est accepté
#[test]
fn selection_mode_requires_selection_and_nothing_else() {
    let run = edit(SELECTION, json!({"selection": "Texte plus court."}));
    assert_eq!(run.state, ToolState::Ok);
    let proposal = run.proposal.unwrap();
    assert_eq!(proposal.selection.as_deref(), Some("Texte plus court."));
    assert!(proposal.body.is_none());
    for arguments in [
        json!({"body": "x"}),
        json!({"selection": "x", "subject": "y"}),
    ] {
        assert_eq!(edit(SELECTION, arguments).state, ToolState::Error);
    }
}

/// Sans sélection, le texte de remplacement est interdit
#[test]
fn selection_is_refused_without_a_selection_mode() {
    let run = edit(WHOLE, json!({"selection": "x"}));
    assert_eq!(run.state, ToolState::Error);
    assert!(run.proposal.is_none());
}

/// Un outil hors de son mode est indisponible et ne propose rien
#[test]
fn tools_outside_their_mode_are_unavailable() {
    let compose = call("compose_draft", json!({"body": "x"}));
    for mode in [WHOLE, SELECTION] {
        let run = propose(mode, &compose);
        assert_eq!(run.state, ToolState::Error);
        assert!(run.proposal.is_none());
        assert!(run.text.contains("indisponible"));
    }
    let run = propose(
        ChatMode::Assistant,
        &call("edit_draft", json!({"body": "x"})),
    );
    assert_eq!(run.state, ToolState::Error);
    assert!(run.proposal.is_none());
}

/// Des arguments illisibles ou une adresse invalide sont expliqués au modèle
#[test]
fn unreadable_arguments_and_bad_recipients_are_errors() {
    let broken = mira_vocabulary::ToolCall {
        id: "c1".into(),
        name: "edit_draft".into(),
        arguments: "pas du json".into(),
    };
    assert_eq!(propose(WHOLE, &broken).state, ToolState::Error);
    let bad = edit(WHOLE, json!({"to": ["nope"]}));
    assert_eq!(bad.state, ToolState::Error);
    assert!(bad.proposal.is_none());
}
