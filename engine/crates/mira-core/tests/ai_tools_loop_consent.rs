#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Boucle d'outils, consentement : décision par type de contenu, connexion locale exemptée.

mod ai_loop_support;
mod ai_models_support;
mod support;
mod support_api;
mod support_mirror;
mod support_read;

use ai_loop_support::fixture::LoopFixture;
use ai_loop_support::turns::{FORMATS, collect, shape, text_turn, tool_turn};
use mira_core::ai::chat::{ToolDecision, decide_tool};
use mira_vocabulary::ContentKind;

/// Un outil inconnu n'a pas de type, donc rien à autoriser
#[test]
fn unknown_tool_is_unknown() {
    assert_eq!(decide_tool(false, &[], None), ToolDecision::Unknown);
}

/// Une connexion distante sans le type autorisé refuse, et nomme le type refusé
#[test]
fn remote_connection_denies_a_kind_not_allowed() {
    let allowed = [ContentKind::MailMessage];
    let decision = decide_tool(false, &allowed, Some(ContentKind::MailList));
    assert_eq!(decision, ToolDecision::Denied(ContentKind::MailList));
}

/// Une connexion distante exécute un type autorisé
#[test]
fn remote_connection_runs_an_allowed_kind() {
    let allowed = [ContentKind::MailList];
    let decision = decide_tool(false, &allowed, Some(ContentKind::MailList));
    assert_eq!(decision, ToolDecision::Run(ContentKind::MailList));
}

/// Une connexion neuve, sans aucun type autorisé, refuse tout
#[test]
fn fresh_remote_connection_denies_everything() {
    for kind in [
        ContentKind::MailList,
        ContentKind::MailMessage,
        ContentKind::ToolResult,
    ] {
        assert_eq!(
            decide_tool(false, &[], Some(kind)),
            ToolDecision::Denied(kind)
        );
    }
}

/// La boucle locale est exemptée : l'outil s'exécute sans aucun type autorisé
#[test]
fn loopback_connection_is_exempt() {
    let decision = decide_tool(true, &[], Some(ContentKind::MailList));
    assert_eq!(decision, ToolDecision::Run(ContentKind::MailList));
}

/// De bout en bout sur la boucle locale : l'outil s'exécute sans type autorisé
#[tokio::test]
async fn local_connection_runs_tools_without_allowed_kinds() {
    for format in FORMATS {
        let fixture = LoopFixture::start(format).await;
        assert!(fixture.connection.allowed_kinds.is_empty());
        fixture.queue(tool_turn(
            format,
            "",
            &[("c1", "list_mailboxes", "{}")],
            None,
        ));
        fixture.queue(text_turn(format, "Voilà.", None));
        let deltas = collect(&mut fixture.ask("Liste").await).await;
        assert!(
            shape(&deltas).contains(&"finished:ok".to_string()),
            "{format:?}"
        );
    }
}
