//! Outils en lecture seule : les lignes `tool_called` et `tool_finished` suivent le contrat, sans `allow_tools` il n'y en a pas.

use mira_testkit::ai::{openai_stream, openai_tool_turn};

use super::chat_fixture::{chat_json, chat_json_tools};
use super::chat_start::{collect_lines, start};
use super::driver::Driver;

/// Types des lignes reçues, dans l'ordre.
fn kinds(lines: &[serde_json::Value]) -> Vec<&str> {
    lines.iter().map(|l| l["type"].as_str().unwrap()).collect()
}

/// Avec les outils permis, l'appel est annoncé puis terminé entre `started` et le texte final
pub async fn tool_lines_arrive_in_order<D: Driver>() {
    let f = start::<D>().await;
    f.server.queue(openai_tool_turn(
        "",
        &[("c1", "list_mailboxes", "{}")],
        None,
    ));
    f.server.queue(openai_stream(&["Voilà."], None));
    let request = chat_json_tools(Some(&f.connection), "Liste mes boîtes");
    let lines = collect_lines(&f.driver, &request).await;
    let expected = ["started", "tool_called", "tool_finished", "text", "done"];
    assert_eq!(kinds(&lines), expected, "{lines:?}");
    assert_eq!(lines[1]["name"], "list_mailboxes");
    assert_eq!(lines[1]["id"], "c1");
    assert_eq!(lines[2]["state"], "ok");
    assert_eq!(lines[2]["sources"], serde_json::json!([]));
    assert!(f.driver.close().await.contains("\"ok\":true"));
}

/// Sans `allow_tools`, aucune ligne d'outil et aucun outil annoncé au modèle
pub async fn start_without_allow_tools_has_no_tool_lines<D: Driver>() {
    let f = start::<D>().await;
    f.server.queue(openai_stream(&["Bonjour."], None));
    let lines = collect_lines(&f.driver, &chat_json(Some(&f.connection), "Salut")).await;
    assert_eq!(kinds(&lines), ["started", "text", "done"], "{lines:?}");
    assert!(!f.server.requests()[0].body.contains("\"tools\""));
    assert!(f.driver.close().await.contains("\"ok\":true"));
}
