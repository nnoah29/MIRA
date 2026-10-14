//! Un exemple sérialisé par variante de `Request` : la liste à compléter quand une variante naît.

use mira_assembly::api::Request;

const DRAFT: &str =
    r#"{"address":"a@b.c","display_name":"n","provider":null,"servers":null,"auth":"password"}"#;
const CREDENTIAL: &str = r#"{"type":"password","value":"x"}"#;

/// Exemples des domaines moteur, providers, identités, authentification, opérations, événements.
const SIMPLE: &[&str] = &[
    r#"{"domain":"engine","request":{"op":"get_contract_info"}}"#,
    r#"{"domain":"engine","request":{"op":"get_status"}}"#,
    r#"{"domain":"provider","request":{"op":"list_providers"}}"#,
    r#"{"domain":"provider","request":{"op":"detect_provider","address":"a@b.c"}}"#,
    r#"{"domain":"identity","request":{"op":"list_identities","account":"a"}}"#,
    r#"{"domain":"identity","request":{"op":"add_identity","account":"a","address":"a@b.c","display_name":"n","signature":null,"is_default":false}}"#,
    r#"{"domain":"identity","request":{"op":"update_identity","identity":"i","display_name":null,"signature":null}}"#,
    r#"{"domain":"identity","request":{"op":"set_default_identity","identity":"i"}}"#,
    r#"{"domain":"auth","request":{"op":"start_oauth","address":"a@b.c","provider":null}}"#,
    r#"{"domain":"auth","request":{"op":"complete_oauth","flow":"f"}}"#,
    r#"{"domain":"auth","request":{"op":"cancel_oauth","flow":"f"}}"#,
    r#"{"domain":"auth","request":{"op":"store_app_password","account":"a","password":"x"}}"#,
    r#"{"domain":"auth","request":{"op":"reauthenticate","account":"a"}}"#,
    r#"{"domain":"operation","request":{"op":"get_operation","operation":"o"}}"#,
    r#"{"domain":"operation","request":{"op":"cancel_operation","operation":"o"}}"#,
    r#"{"domain":"event","request":{"op":"get_events","filter":{"accounts":[],"categories":[],"after":null},"limit":10}}"#,
];

/// Texte JSON d'une requête du domaine des comptes.
fn account_json(body: &str) -> String {
    format!(r#"{{"domain":"account","request":{{{body}}}}}"#)
}

/// Exemples du domaine des comptes.
fn account_examples() -> Vec<String> {
    vec![
        account_json(r#""op":"prepare_account","address":"a@b.c","provider":null"#),
        account_json(&format!(
            r#""op":"test_connection","draft":{DRAFT},"credential":{CREDENTIAL}"#
        )),
        account_json(&format!(
            r#""op":"add_account","draft":{DRAFT},"credential":{CREDENTIAL},"idempotency_key":"k""#
        )),
        account_json(r#""op":"list_accounts""#),
        account_json(
            r#""op":"update_account","account":"a","changes":{"display_name":null,"servers":null,"window_days":null,"synced_mailboxes":null}"#,
        ),
        account_json(r#""op":"pause_account","account":"a""#),
        account_json(r#""op":"resume_account","account":"a""#),
        account_json(r#""op":"delete_account","account":"a""#),
    ]
}

/// Toutes les requêtes d'exemple, une par variante connue.
pub fn all_examples() -> Vec<Request> {
    let mut texts: Vec<String> = SIMPLE.iter().map(ToString::to_string).collect();
    texts.extend(account_examples());
    texts.extend(super::examples_read::mailbox_and_thread_examples());
    texts.extend(super::examples_read::search_examples());
    texts.extend(super::examples_read::maintenance_examples());
    texts.extend(super::examples_read::message_examples());
    texts.extend(super::examples_sync::sync_examples());
    texts.extend(super::examples_write::action_and_queue_examples());
    texts.extend(super::examples_write::draft_examples());
    texts.extend(super::examples_write::send_examples());
    texts
        .iter()
        .map(|text| serde_json::from_str(text).unwrap())
        .collect()
}
