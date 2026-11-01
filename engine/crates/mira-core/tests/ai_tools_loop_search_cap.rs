#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Boucle d'outils : après quatre tours, plus d'outil de lecture annoncé, seulement ceux qui écrivent ; un outil retiré est refusé.

mod ai_loop_support;
mod ai_models_support;
mod support;
mod support_api;
mod support_mirror;
mod support_read;

use ai_loop_support::fixture::LoopFixture;
use ai_loop_support::turns::{collect, shape, text_turn, tool_turn};
use mira_vocabulary::WireFormat;
use serde_json::Value;

const SEARCH: &str = r#"{"query":"boss"}"#;
const FORMAT: WireFormat = WireFormat::OpenAiChat;

/// Noms des outils annoncés dans un corps de requête au format OpenAI
fn announced(body: &Value) -> Vec<String> {
    let tools = body["tools"].as_array().cloned().unwrap_or_default();
    let name = |tool: &Value| {
        tool.pointer("/function/name")
            .and_then(Value::as_str)
            .map(str::to_string)
    };
    tools.iter().filter_map(name).collect()
}

/// Quatre tours de recherches, puis un modèle qui rappelle une recherche malgré tout
async fn four_rounds_then(last: &str) -> (LoopFixture, Vec<String>) {
    let fixture = LoopFixture::start(FORMAT).await;
    for n in 0..4 {
        fixture.queue(tool_turn(
            FORMAT,
            "",
            &[(&format!("c{n}"), "search_messages", SEARCH)],
            None,
        ));
    }
    if last == "search" {
        fixture.queue(tool_turn(
            FORMAT,
            "",
            &[("c9", "search_messages", SEARCH)],
            None,
        ));
    }
    fixture.queue(text_turn(FORMAT, "Voilà.", None));
    let mut stream = fixture.ask("écris à mon boss").await;
    let found = shape(&collect(&mut stream).await);
    (fixture, found)
}

/// Dès le cinquième appel au modèle, seuls les outils qui écrivent restent annoncés ; les quatre premiers voient la lecture
#[tokio::test]
async fn after_four_rounds_only_the_writing_tools_remain() {
    let (fixture, _) = four_rounds_then("text").await;
    let bodies = fixture.bodies();
    assert!(announced(&bodies[3]).contains(&"search_messages".to_string()));
    let last = announced(&bodies[4]);
    assert!(!last.contains(&"search_messages".to_string()), "{last:?}");
    assert!(last.contains(&"compose_draft".to_string()), "{last:?}");
    assert!(
        last.contains(&"propose_mailbox_action".to_string()),
        "{last:?}"
    );
}

/// Un outil retiré, appelé quand même, est refusé sans rien lire
#[tokio::test]
async fn a_removed_tool_called_anyway_is_refused() {
    let (fixture, found) = four_rounds_then("search").await;
    let refused = found
        .iter()
        .rposition(|label| label == "called:search_messages")
        .unwrap();
    assert_eq!(found[refused + 1], "finished:error", "{found:?}");
    let results = ai_loop_support::turns::tool_results(&fixture.bodies()[5], FORMAT);
    assert!(
        results.last().unwrap().contains("indisponible"),
        "{results:?}"
    );
}
