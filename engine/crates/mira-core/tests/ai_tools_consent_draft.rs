#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Consentement et outils de brouillon : une proposition n'exige aucun type autorisé, la lecture d'identités oui.

use mira_core::ai::chat::{ToolDecision, decide_call};
use mira_vocabulary::ContentKind;

/// Sur une connexion distante sans aucun type autorisé, une proposition passe
#[test]
fn proposal_tools_need_no_authorized_kind() {
    for name in ["compose_draft", "edit_draft", "propose_mailbox_action"] {
        assert_eq!(
            decide_call(false, &[], name),
            ToolDecision::Proposal,
            "{name}"
        );
    }
}

/// Lire les identités suit le consentement : refusé sans le type, permis avec, exempté en local
#[test]
fn list_identities_follows_the_consent() {
    let denied = decide_call(false, &[], "list_identities");
    assert_eq!(denied, ToolDecision::Denied(ContentKind::ToolResult));
    let allowed = decide_call(false, &[ContentKind::ToolResult], "list_identities");
    assert_eq!(allowed, ToolDecision::Run(ContentKind::ToolResult));
    let local = decide_call(true, &[], "list_identities");
    assert_eq!(local, ToolDecision::Run(ContentKind::ToolResult));
}

/// Un nom inconnu reste inconnu, même en local
#[test]
fn unknown_names_stay_unknown() {
    assert_eq!(
        decide_call(true, &[], "delete_everything"),
        ToolDecision::Unknown
    );
    assert_eq!(
        decide_call(false, &[], "delete_everything"),
        ToolDecision::Unknown
    );
}
