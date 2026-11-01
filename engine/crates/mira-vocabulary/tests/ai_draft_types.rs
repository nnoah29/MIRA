#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Modes de génération, propositions de brouillon et delta qui les porte.

use mira_vocabulary::{AddressEntry, ChatDelta, ChatMode, DraftProposal, DraftProposalKind};
use serde_json::json;

/// Une proposition d'exemple : un nouveau message complet
fn sample() -> DraftProposal {
    DraftProposal {
        kind: DraftProposalKind::New,
        to: Some(vec![AddressEntry {
            name: Some("Ma boss".into()),
            address: "boss@example.org".into(),
        }]),
        cc: None,
        subject: Some("Fin de stage".into()),
        body: Some("Bonjour,\n\nMon stage se termine mercredi.".into()),
        selection: None,
        reply_to: Some("m-1".into()),
        identity: None,
    }
}

/// Sans champ `mode`, une demande est une discussion d'assistant
#[test]
fn chat_mode_defaults_to_assistant() {
    assert_eq!(ChatMode::default(), ChatMode::Assistant);
}

/// Le mode s'écrit avec une étiquette `kind` et se relit à l'identique
#[test]
fn chat_mode_serializes_with_a_kind_tag() {
    let assistant = serde_json::to_value(ChatMode::Assistant).unwrap();
    assert_eq!(assistant, json!({"kind": "assistant"}));
    let composer = ChatMode::Composer { selection: true };
    let value = serde_json::to_value(composer).unwrap();
    assert_eq!(value, json!({"kind": "composer", "selection": true}));
    let back: ChatMode = serde_json::from_value(value).unwrap();
    assert_eq!(back, composer);
}

/// Une proposition survit à un aller-retour JSON et écrit ses genres en `snake_case`
#[test]
fn draft_proposal_round_trips() {
    let value = serde_json::to_value(sample()).unwrap();
    assert_eq!(value["kind"], "new");
    assert_eq!(value["reply_to"], "m-1");
    let back: DraftProposal = serde_json::from_value(value).unwrap();
    assert_eq!(back, sample());
    let edit = serde_json::to_value(DraftProposalKind::Edit).unwrap();
    assert_eq!(edit, json!("edit"));
}

/// Le delta de proposition porte l'identifiant de l'appel et la proposition
#[test]
fn draft_proposed_delta_has_snake_case_type() {
    let delta = ChatDelta::DraftProposed {
        id: "call-1".into(),
        proposal: sample(),
    };
    let value = serde_json::to_value(&delta).unwrap();
    assert_eq!(value["type"], "draft_proposed");
    assert_eq!(value["id"], "call-1");
    assert_eq!(value["proposal"]["subject"], "Fin de stage");
    let back: ChatDelta = serde_json::from_value(value).unwrap();
    assert_eq!(back, delta);
}
