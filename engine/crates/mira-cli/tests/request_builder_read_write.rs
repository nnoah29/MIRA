#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Lignes de commande de lecture et d'écriture : la requête du contrat qu'elles construisent, au champ près.

#[allow(dead_code)]
mod cli_support;
#[allow(dead_code)]
mod request_catalog;

use cli_support::json_views::words;
use request_catalog::{json_of_line as json_of, try_request_of_line};
use serde_json::json;

/// `message list` : vue, filtres, dates et page sont traduits tels quels
#[test]
fn message_list_line_builds_the_expected_query() {
    let line = [
        "message",
        "list",
        "--mailbox",
        "box",
        "--unread",
        "--from",
        "bob",
        "--since",
        "2023-11-14",
        "--limit",
        "7",
        "--cursor",
        "c",
    ];
    let filters = json!({"unread_only": true, "flagged_only": false, "has_attachments": false,
        "from_contains": "bob", "since": 1_699_920_000_000_i64, "until": null});
    assert_eq!(
        json_of(&line),
        json!({"domain": "message", "request": {"op": "list_messages",
            "view": {"type": "mailbox", "mailbox": "box"}, "filters": filters,
            "page": {"cursor": "c", "limit": 7}}})
    );
}

/// `action apply` : nature, cibles par liste ou par requête, clé fournie ou fabriquée
#[test]
fn action_apply_lines_build_kinds_targets_and_keys() {
    let listed = json_of(&words(
        "action apply mark-unread --message a --message b --key k",
    ));
    let request = &listed["request"];
    assert_eq!(request["kind"], json!({"type": "mark_read", "read": false}));
    assert_eq!(
        request["targets"],
        json!({"type": "messages", "messages": ["a", "b"]})
    );
    assert_eq!(request["idempotency_key"], "k");
    let queried = json_of(&words("action apply move --view flagged --to arch"));
    assert_eq!(
        queried["request"]["kind"],
        json!({"type": "move", "to": "arch"})
    );
    assert_eq!(queried["request"]["targets"]["type"], "query");
    assert_eq!(
        queried["request"]["targets"]["query"]["view"],
        json!({"type": "standard", "view": "flagged"})
    );
    assert_eq!(queried["request"]["idempotency_key"], "cli-id-000001");
}

/// `draft update` : destinataires `Nom <adresse>`, liste vidée par une valeur vide, champs absents inchangés
#[test]
fn draft_update_line_builds_recipients_and_partial_changes() {
    let line = [
        "draft",
        "update",
        "d",
        "--to",
        "Bob <bob@example.org>",
        "--to",
        "c@x.org",
        "--cc",
        "",
        "--html",
        "",
    ];
    let changes = &json_of(&line)["request"]["changes"];
    assert_eq!(
        changes["to"][0],
        json!({"name": "Bob", "address": "bob@example.org"})
    );
    assert_eq!(
        changes["to"][1],
        json!({"name": null, "address": "c@x.org"})
    );
    assert_eq!(changes["cc"], json!([]));
    assert_eq!(changes["body_html"], "");
    assert!(
        changes["bcc"].is_null() && changes["subject"].is_null() && changes["body_text"].is_null()
    );
}

/// Boîtes, file et recherche : champs optionnels, états répétés, mots joints
#[test]
fn mailbox_queue_and_search_lines_build_their_fields() {
    let created = json_of(&["mailbox", "create", "acc", "Projets", "--parent", "p"]);
    assert_eq!(created["request"]["parent"], "p");
    assert_eq!(created["request"]["idempotency_key"], "cli-id-000001");
    let states = json_of(&words("queue list acc --state pending --state in-progress"));
    assert_eq!(
        states["request"]["states"],
        json!(["pending", "in_progress"])
    );
    let search = json_of(&words("search local rapport annuel --to bob --mailbox m"));
    assert_eq!(search["request"]["query"], "rapport annuel");
    assert_eq!(search["request"]["filters"]["to_contains"], "bob");
    assert_eq!(search["request"]["filters"]["mailbox"], "m");
}

/// Instants : millisecondes, date, date et heure UTC ; formes invalides refusées
#[test]
fn instants_accept_three_forms_and_refuse_garbage() {
    let at = |text: &str| json_of(&["draft", "send", "d", "--at", text])["request"]["at"].clone();
    assert_eq!(at("1700000000000"), 1_700_000_000_000_i64);
    assert_eq!(at("2023-11-14T22:13:20Z"), 1_700_000_000_000_i64);
    assert_eq!(at("2023-11-14"), 1_699_920_000_000_i64);
    for bad in ["2023-13-01", "2023-11-14T25:00", "hier", "2023-11"] {
        let error = try_request_of_line(&["draft", "send", "d", "--at", bad]).unwrap_err();
        assert_eq!(error.code, "invalid_input.parameter", "{bad}");
    }
}

/// Lignes incohérentes : erreur d'entrée invalide avant tout appel au moteur
#[test]
fn inconsistent_lines_are_invalid_input() {
    let lines: [&[&str]; 6] = [
        &["action", "apply", "move", "--message", "m"],
        &["action", "apply", "archive"],
        &["message", "list"],
        &["message", "allow-remote"],
        &["draft", "update", "d", "--to", "pas-une-adresse"],
        &[
            "draft",
            "update",
            "d",
            "--body",
            "a",
            "--body-file",
            "/tmp/x",
        ],
    ];
    for line in lines {
        let error = try_request_of_line(line).unwrap_err();
        assert_eq!(
            error.category,
            mira_assembly::mira_vocabulary::ErrorCategory::InvalidInput,
            "{line:?}"
        );
    }
}
