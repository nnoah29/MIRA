#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Forme JSON des trames : chaque type documenté fait l'aller-retour sans perte.

use std::collections::BTreeSet;

use mira_protocol::{EndReason, Frame, HelloRefusal};
use serde_json::{Value, json};

const VERSIONS: &str =
    r#""protocol_version":{"major":1,"minor":0},"contract_version":{"major":1,"minor":6}"#;

const NAMES: [&str; 16] = [
    "hello",
    "hello_ok",
    "hello_err",
    "call",
    "result",
    "open_events",
    "open_chat",
    "get_original_html",
    "host_op",
    "event",
    "chat_delta",
    "credit",
    "cancel",
    "end",
    "ping",
    "pong",
];

/// Un exemple JSON de chaque trame, tel que le client Dart l'écrira.
fn examples() -> Vec<Value> {
    let texts = [
        format!(
            r#"{{"type":"hello",{VERSIONS},"client_id":"c1","client_name":"app","auth":{{"type":"local_token","token":"t"}}}}"#
        ),
        format!(r#"{{"type":"hello_ok",{VERSIONS},"host_state":"ready"}}"#),
        r#"{"type":"hello_err","reason":"contract_version"}"#.to_string(),
        r#"{"type":"call","id":1,"request":{"domain":"account"}}"#.to_string(),
        r#"{"type":"result","id":1,"envelope":{"ok":true}}"#.to_string(),
        r#"{"type":"open_events","id":2,"after":null,"credit":8}"#.to_string(),
        r#"{"type":"open_chat","id":3,"request":{"mode":"x"},"credit":8}"#.to_string(),
        r#"{"type":"get_original_html","id":4,"message":"m1"}"#.to_string(),
        r#"{"type":"host_op","id":5,"op":"shutdown"}"#.to_string(),
        r#"{"type":"event","id":2,"event":{"seq":1}}"#.to_string(),
        r#"{"type":"chat_delta","id":3,"line":{"type":"started"}}"#.to_string(),
        r#"{"type":"credit","id":2,"n":4}"#.to_string(),
        r#"{"type":"cancel","id":3}"#.to_string(),
        r#"{"type":"end","id":2,"reason":{"type":"lagged"}}"#.to_string(),
        r#"{"type":"ping","nonce":7}"#.to_string(),
        r#"{"type":"pong","nonce":7}"#.to_string(),
    ];
    texts
        .iter()
        .map(|t| serde_json::from_str(t).unwrap())
        .collect()
}

/// Chaque exemple se lit en trame puis se réécrit à l'identique.
#[test]
fn every_frame_round_trips_through_json() {
    for example in examples() {
        let frame: Frame = serde_json::from_value(example.clone()).unwrap();
        assert_eq!(serde_json::to_value(&frame).unwrap(), example);
    }
}

/// Les 16 noms de `type` documentés sont exactement ceux des exemples.
#[test]
fn the_type_tag_uses_the_documented_names() {
    let seen: BTreeSet<String> = examples()
        .iter()
        .map(|e| e["type"].as_str().unwrap().to_string())
        .collect();
    let documented: BTreeSet<String> = NAMES.iter().map(|n| n.to_string()).collect();
    assert_eq!(seen, documented);
}

/// Le refus de poignée de main ne porte qu'une raison fixe, jamais de texte libre.
#[test]
fn hello_err_carries_only_a_fixed_reason() {
    let frame = Frame::HelloErr {
        reason: HelloRefusal::Auth,
    };
    let expected = json!({"type": "hello_err", "reason": "auth"});
    assert_eq!(serde_json::to_value(&frame).unwrap(), expected);
}

/// Un type inconnu, ou une raison de fin inconnue, est refusé à la lecture.
#[test]
fn an_unknown_type_is_rejected() {
    let unknown = json!({"type": "teleport", "id": 1});
    assert!(serde_json::from_value::<Frame>(unknown).is_err());
    let reason = json!({"type": "mystery"});
    assert!(serde_json::from_value::<EndReason>(reason).is_err());
}
