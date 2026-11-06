//! Brouillons et actions proposés par l'IA : les lignes `draft_proposed` et `action_proposed` suivent le contrat.

use mira_assembly::mira_vocabulary::ChatMode;
use mira_testkit::ai::{openai_stream, openai_tool_turn};

use super::chat_fixture::chat_json_mode;
use super::chat_start::{collect_lines, start};
use super::driver::Driver;

const COMPOSE: &str = r#"{"to":["Boss <boss@example.org>"],"subject":"Fin de stage","body":"Bonjour, mon stage finit mercredi."}"#;

/// Types des lignes reçues, dans l'ordre.
fn kinds(lines: &[serde_json::Value]) -> Vec<&str> {
    lines.iter().map(|l| l["type"].as_str().unwrap()).collect()
}

/// La ligne `action_proposed` porte l'appel, l'action et les messages visés.
fn check_action_proposal(line: &serde_json::Value) {
    assert_eq!(line["id"], "c1");
    assert_eq!(line["proposal"]["action"], "archive");
    assert_eq!(
        line["proposal"]["messages"],
        serde_json::json!(["m1", "m2"])
    );
}

/// En mode assistant, la proposition arrive entre l'annonce et la fin de l'appel
pub async fn draft_proposed_line_arrives<D: Driver>() {
    let f = start::<D>().await;
    f.server.queue(openai_tool_turn(
        "",
        &[("c1", "compose_draft", COMPOSE)],
        None,
    ));
    f.server.queue(openai_stream(&["Voilà."], None));
    let json = chat_json_mode(Some(&f.connection), "écris à mon boss", ChatMode::Assistant);
    let lines = collect_lines(&f.driver, &json).await;
    let expected = [
        "started",
        "tool_called",
        "draft_proposed",
        "tool_finished",
        "text",
        "done",
    ];
    assert_eq!(kinds(&lines), expected, "{lines:?}");
    assert_eq!(lines[2]["id"], "c1");
    assert_eq!(lines[2]["proposal"]["kind"], "new");
    assert_eq!(lines[2]["proposal"]["subject"], "Fin de stage");
    assert_eq!(lines[2]["proposal"]["to"][0]["address"], "boss@example.org");
    assert!(f.driver.close().await.contains("\"ok\":true"));
}

/// Le mode composeur arrive du JSON de l'app : `edit_draft` est annoncé, `compose_draft` non
pub async fn composer_mode_comes_from_the_json<D: Driver>() {
    let f = start::<D>().await;
    let args = r#"{"body":"Nouveau"}"#;
    f.server
        .queue(openai_tool_turn("", &[("c1", "edit_draft", args)], None));
    f.server.queue(openai_stream(&["Fait."], None));
    let mode = ChatMode::Composer { selection: false };
    let json = chat_json_mode(Some(&f.connection), "raccourcis", mode);
    assert!(
        json.contains(r#""mode":{"kind":"composer","selection":false}"#),
        "{json}"
    );
    let lines = collect_lines(&f.driver, &json).await;
    assert_eq!(lines[2]["type"], "draft_proposed", "{lines:?}");
    assert_eq!(lines[2]["proposal"]["kind"], "edit");
    let body = &f.server.requests()[0].body;
    assert!(
        body.contains("edit_draft") && !body.contains("compose_draft"),
        "{body}"
    );
    assert!(f.driver.close().await.contains("\"ok\":true"));
}

/// Une action sur la boîte proposée arrive comme ligne `action_proposed`, sans rien modifier
pub async fn action_proposed_line_arrives<D: Driver>() {
    let f = start::<D>().await;
    let args = r#"{"action":"archive","messages":["m1","m2"],"reason":"Newsletters lues"}"#;
    f.server.queue(openai_tool_turn(
        "",
        &[("c1", "propose_mailbox_action", args)],
        None,
    ));
    f.server
        .queue(openai_stream(&["À toi de confirmer."], None));
    let json = chat_json_mode(
        Some(&f.connection),
        "archive mes newsletters",
        ChatMode::Assistant,
    );
    let lines = collect_lines(&f.driver, &json).await;
    let expected = [
        "started",
        "tool_called",
        "action_proposed",
        "tool_finished",
        "text",
        "done",
    ];
    assert_eq!(kinds(&lines), expected, "{lines:?}");
    check_action_proposal(&lines[2]);
    assert!(f.driver.close().await.contains("\"ok\":true"));
}
