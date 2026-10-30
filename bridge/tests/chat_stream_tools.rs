#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Pont `chat_stream` et outils : les lignes `tool_called` et `tool_finished` suivent le contrat, sans `allow_tools` il n'y en a pas.

mod support;

use mira_bridge::runtime;
use mira_testkit::ai::{openai_stream, openai_tool_turn};
use support::chat_fixture::{chat_json, chat_json_tools, collect_lines, start};

/// Types des lignes reçues, dans l'ordre.
fn kinds(lines: &[serde_json::Value]) -> Vec<&str> {
    lines.iter().map(|l| l["type"].as_str().unwrap()).collect()
}

/// Avec les outils permis, l'appel est annoncé puis terminé entre `started` et le texte final
#[tokio::test(flavor = "multi_thread")]
async fn tool_lines_reach_the_sink_in_order() {
    let f = start().await;
    f.server.queue(openai_tool_turn(
        "",
        &[("c1", "list_mailboxes", "{}")],
        None,
    ));
    f.server.queue(openai_stream(&["Voilà."], None));
    let lines = collect_lines(&chat_json_tools(Some(&f.connection), "Liste mes boîtes")).await;
    let expected = ["started", "tool_called", "tool_finished", "text", "done"];
    assert_eq!(kinds(&lines), expected, "{lines:?}");
    assert_eq!(lines[1]["name"], "list_mailboxes");
    assert_eq!(lines[1]["id"], "c1");
    assert_eq!(lines[2]["state"], "ok");
    assert_eq!(lines[2]["sources"], serde_json::json!([]));
    assert!(runtime::close().await.contains("\"ok\":true"));
}

/// Sans `allow_tools`, aucune ligne d'outil et aucun outil annoncé au modèle
#[tokio::test(flavor = "multi_thread")]
async fn start_without_allow_tools_has_no_tool_lines() {
    let f = start().await;
    f.server.queue(openai_stream(&["Bonjour."], None));
    let lines = collect_lines(&chat_json(Some(&f.connection), "Salut")).await;
    assert_eq!(kinds(&lines), ["started", "text", "done"], "{lines:?}");
    assert!(!f.server.requests()[0].body.contains("\"tools\""));
    assert!(runtime::close().await.contains("\"ok\":true"));
}
