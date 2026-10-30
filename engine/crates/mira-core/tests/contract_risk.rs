#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Niveau de risque de chaque requête du contrat : seul l'ensemble exact des lectures est exposable aux outils.

use std::path::PathBuf;

use mira_core::api::Request;
use mira_core::api::contract::risk_of;
use mira_vocabulary::ToolRisk;
use serde_json::Value;

const READS: &[&str] = &[
    "account.list_accounts",
    "ai.list_ai_connections",
    "ai.list_ai_journal",
    "ai.list_ai_providers",
    "compose.get_draft",
    "compose.list_drafts",
    "compose.list_outbox",
    "engine.get_contract_info",
    "engine.get_status",
    "event.get_events",
    "identity.list_identities",
    "label_settings.list_labels",
    "local_label.export_local_labels_file",
    "local_label.list_local_label_assignments",
    "local_label.list_local_labels",
    "mailbox.list_mailboxes",
    "maintenance.check_integrity",
    "maintenance.get_storage_stats",
    "message.get_message",
    "message.get_message_body",
    "message.get_raw_source",
    "message.get_safe_html",
    "message.list_attachments",
    "message.list_messages",
    "operation.get_operation",
    "provider.list_providers",
    "queue.list_actions",
    "rule.export_rules",
    "rule.get_rule",
    "rule.list_rule_catalog",
    "rule.list_rules",
    "rule.preview_rule",
    "rule.validate_rule",
    "search.get_server_search_result",
    "search.search",
    "settings.get_settings",
    "sync.get_sync_status",
    "system_label.list_system_label_assignments",
    "template.get_template",
    "template.list_templates",
    "template.render_template",
    "thread.get_thread",
    "thread.list_threads",
];

/// Toutes les requêtes d'exemple du contrat, relues depuis l'instantané versionné.
fn requests() -> Vec<(String, Request)> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots/contract_v1.json");
    let text = std::fs::read_to_string(path).unwrap();
    let snapshot: Value = serde_json::from_str(&text).unwrap();
    let all = snapshot["requests"].as_object().unwrap();
    all.iter()
        .map(|(name, value)| (name.clone(), serde_json::from_value(value.clone()).unwrap()))
        .collect()
}

/// Niveau de la requête d'exemple de ce nom.
fn risk_named(name: &str) -> ToolRisk {
    let found = requests().into_iter().find(|(n, _)| n == name);
    risk_of(&found.unwrap().1)
}

/// Les lectures sont exactement l'ensemble attendu, ni plus ni moins
#[test]
fn read_set_is_exactly_the_expected_names() {
    let mut reads: Vec<String> = requests()
        .into_iter()
        .filter(|(_, request)| risk_of(request) == ToolRisk::Read)
        .map(|(name, _)| name.split('.').take(2).collect::<Vec<_>>().join("."))
        .collect();
    reads.dedup();
    assert_eq!(reads, READS);
}

/// Une opération réseau lancée par le modèle n'est jamais une lecture
#[test]
fn search_server_and_sync_are_not_read() {
    assert_ne!(risk_named("search.search_server"), ToolRisk::Read);
    assert_ne!(risk_named("sync.start_sync"), ToolRisk::Read);
}

/// Envoyer, supprimer et configurer ont des niveaux distincts et prudents
#[test]
fn sending_deleting_and_configuring_have_cautious_levels() {
    assert_eq!(risk_named("compose.send_draft"), ToolRisk::Confirmed);
    assert_eq!(risk_named("compose.update_draft"), ToolRisk::Reversible);
    assert_eq!(risk_named("account.delete_account"), ToolRisk::Sensitive);
    assert_eq!(risk_named("ai.add_ai_connection"), ToolRisk::Sensitive);
}
