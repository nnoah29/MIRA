//! Exemples sérialisés des variantes du domaine synchronisation.

/// Texte JSON d'une requête du domaine synchronisation.
fn sync_json(body: &str) -> String {
    format!(r#"{{"domain":"sync","request":{{{body}}}}}"#)
}

/// Un exemple par variante de `SyncRequest`, plus les trois portées de `StartSync`.
pub fn sync_examples() -> Vec<String> {
    vec![
        sync_json(r#""op":"start_sync","scope":{"type":"all"}"#),
        sync_json(r#""op":"start_sync","scope":{"type":"account","account":"a"}"#),
        sync_json(r#""op":"start_sync","scope":{"type":"mailbox","mailbox":"m"}"#),
        sync_json(r#""op":"pause_sync","account":"a""#),
        sync_json(r#""op":"get_sync_status""#),
        sync_json(r#""op":"extend_window","mailbox":"m","days":30"#),
        sync_json(r#""op":"resync_mailbox","mailbox":"m""#),
    ]
}
