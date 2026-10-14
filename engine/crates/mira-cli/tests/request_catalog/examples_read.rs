//! Exemples sérialisés des variantes des domaines de lecture et de maintenance.

const PAGE: &str = r#"{"cursor":null,"limit":10}"#;
const FILTERS: &str = r#"{"unread_only":false,"flagged_only":false,"has_attachments":false,"from_contains":null,"since":null,"until":null}"#;

/// Texte JSON d'une requête d'un domaine.
fn request_json(domain: &str, body: &str) -> String {
    format!(r#"{{"domain":"{domain}","request":{{{body}}}}}"#)
}

/// Exemples des domaines boîtes et fils.
pub fn mailbox_and_thread_examples() -> Vec<String> {
    let threads = format!(r#""op":"list_threads","mailbox":"m","page":{PAGE}"#);
    vec![
        request_json("mailbox", r#""op":"list_mailboxes","account":"a""#),
        request_json(
            "mailbox",
            r#""op":"set_synced_mailboxes","account":"a","mailboxes":["m"]"#,
        ),
        request_json("thread", &threads),
        request_json("thread", r#""op":"get_thread","thread":"t""#),
    ]
}

/// Exemples du domaine recherche.
pub fn search_examples() -> Vec<String> {
    let search = format!(
        r#""op":"search","query":"q","filters":{{"common":{FILTERS},"to_contains":null,"mailbox":null}},"page":{PAGE}"#
    );
    vec![
        request_json("search", &search),
        request_json(
            "search",
            r#""op":"search_server","account":"a","query":"q""#,
        ),
        request_json(
            "search",
            r#""op":"get_server_search_result","operation":"o""#,
        ),
    ]
}

/// Exemples du domaine maintenance.
pub fn maintenance_examples() -> Vec<String> {
    vec![
        request_json("maintenance", r#""op":"get_storage_stats""#),
        request_json("maintenance", r#""op":"purge_attachment_cache""#),
        request_json("maintenance", r#""op":"rebuild_search_index""#),
        request_json("maintenance", r#""op":"check_integrity""#),
        request_json(
            "maintenance",
            r#""op":"export_diagnostic","destination":"/tmp/d""#,
        ),
        request_json(
            "maintenance",
            r#""op":"export_profile","destination":"/tmp/p""#,
        ),
    ]
}

/// Exemples du domaine de lecture des messages.
pub fn message_examples() -> Vec<String> {
    let list = format!(
        r#""op":"list_messages","view":{{"type":"mailbox","mailbox":"m"}},"filters":{FILTERS},"page":{PAGE}"#
    );
    let one = |op: &str| request_json("message", &format!(r#""op":"{op}","message":"m""#));
    vec![
        request_json("message", &list),
        one("get_message"),
        one("get_message_body"),
        one("fetch_message_body"),
        one("get_raw_source"),
        one("list_attachments"),
        one("get_safe_html"),
        request_json(
            "message",
            r#""op":"save_attachment","attachment":"x","destination":"/tmp/f""#,
        ),
        request_json(
            "message",
            r#""op":"allow_remote_content","scope":{"type":"sender","address":"a@b.c"}"#,
        ),
    ]
}
