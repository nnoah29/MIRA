//! Exemples sérialisés des variantes des domaines d'écriture : actions, rédaction, file.

/// Texte JSON d'une requête d'un domaine.
fn request_json(domain: &str, body: &str) -> String {
    format!(r#"{{"domain":"{domain}","request":{{{body}}}}}"#)
}

/// Exemples des domaines actions et file d'actions.
pub fn action_and_queue_examples() -> Vec<String> {
    let apply = r#""op":"apply_action","kind":{"type":"mark_read","read":true},"targets":{"type":"messages","messages":["m"]},"idempotency_key":"k""#;
    vec![
        request_json("action", apply),
        request_json("action", r#""op":"undo_action","action":"x""#),
        request_json(
            "action",
            r#""op":"create_mailbox","account":"a","name":"n","parent":null,"idempotency_key":"k""#,
        ),
        request_json(
            "action",
            r#""op":"rename_mailbox","mailbox":"m","name":"n","idempotency_key":"k""#,
        ),
        request_json(
            "action",
            r#""op":"delete_mailbox","mailbox":"m","idempotency_key":"k""#,
        ),
        request_json(
            "action",
            r#""op":"empty_mailbox","mailbox":"m","idempotency_key":"k""#,
        ),
        request_json("queue", r#""op":"list_actions","account":"a","states":[]"#),
        request_json("queue", r#""op":"retry_action","action":"x""#),
        request_json("queue", r#""op":"cancel_action","action":"x""#),
    ]
}

/// Exemples du domaine rédaction : brouillons, pièces jointes, lectures.
pub fn draft_examples() -> Vec<String> {
    let draft = |op: &str| request_json("compose", &format!(r#""op":"{op}","draft":"d""#));
    let account = |op: &str| request_json("compose", &format!(r#""op":"{op}","account":"a""#));
    vec![
        request_json(
            "compose",
            r#""op":"create_draft","account":"a","identity":null,"mode":{"type":"new"}"#,
        ),
        request_json("compose", r#""op":"update_draft","draft":"d","changes":{}"#),
        request_json(
            "compose",
            r#""op":"attach_file","draft":"d","path":"/tmp/f""#,
        ),
        request_json(
            "compose",
            r#""op":"remove_attachment","draft":"d","position":0"#,
        ),
        draft("delete_draft"),
        request_json(
            "compose",
            r#""op":"save_draft_copy","draft":"d","idempotency_key":"k""#,
        ),
        draft("get_draft"),
        account("list_drafts"),
        account("list_outbox"),
    ]
}

/// Exemples du domaine rédaction : envoi, annulation, résolution d'un envoi incertain.
pub fn send_examples() -> Vec<String> {
    vec![
        request_json(
            "compose",
            r#""op":"send_draft","draft":"d","at":null,"idempotency_key":"k""#,
        ),
        request_json("compose", r#""op":"cancel_send","action":"x""#),
        request_json(
            "compose",
            r#""op":"resolve_uncertain_send","action":"x","resend":false"#,
        ),
    ]
}
