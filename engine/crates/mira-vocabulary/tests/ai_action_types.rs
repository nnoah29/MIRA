#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Actions sur la boîte proposées par l'IA : genres, proposition et delta qui la porte.

use mira_vocabulary::{ActionProposal, ChatDelta, MailActionKind};
use serde_json::json;

/// Les six actions s'écrivent en `snake_case` et se relisent
#[test]
fn action_kinds_use_snake_case() {
    let all = [
        (MailActionKind::Archive, "archive"),
        (MailActionKind::Trash, "trash"),
        (MailActionKind::MarkRead, "mark_read"),
        (MailActionKind::MarkUnread, "mark_unread"),
        (MailActionKind::Star, "star"),
        (MailActionKind::Unstar, "unstar"),
    ];
    for (kind, text) in all {
        assert_eq!(serde_json::to_value(kind).unwrap(), json!(text));
        let back: MailActionKind = serde_json::from_value(json!(text)).unwrap();
        assert_eq!(back, kind);
    }
}

/// Une proposition survit à un aller-retour JSON
#[test]
fn action_proposal_round_trips() {
    let proposal = ActionProposal {
        action: MailActionKind::Archive,
        messages: vec!["m1".into(), "m2".into()],
        reason: Some("Newsletters lues".into()),
    };
    let value = serde_json::to_value(&proposal).unwrap();
    assert_eq!(value["action"], "archive");
    assert_eq!(value["messages"], json!(["m1", "m2"]));
    let back: ActionProposal = serde_json::from_value(value).unwrap();
    assert_eq!(back, proposal);
}

/// Le delta porte l'identifiant de l'appel et la proposition
#[test]
fn action_proposed_delta_has_snake_case_type() {
    let delta = ChatDelta::ActionProposed {
        id: "c1".into(),
        proposal: ActionProposal {
            action: MailActionKind::Trash,
            messages: vec!["m9".into()],
            reason: None,
        },
    };
    let value = serde_json::to_value(&delta).unwrap();
    assert_eq!(value["type"], "action_proposed");
    assert_eq!(value["proposal"]["action"], "trash");
    let back: ChatDelta = serde_json::from_value(value).unwrap();
    assert_eq!(back, delta);
}
