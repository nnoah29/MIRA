#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! `mira account add-oauth` : autorisation et ajout dans un seul processus, contre le faux serveur OAuth.

#[allow(dead_code)]
mod cli_support;

use std::time::Duration;

use cli_support::oauth_bench::{OAUTH_USER, OauthBench, run_watching};
use cli_support::parse_json;
use cli_support::shared_buffer::SharedBuffer;
use mira_assembly::mira_vocabulary::SecretKind;

/// Le flux démarre, la redirection arrive, le compte est ajouté avec ses jetons, sans fuite
#[tokio::test]
async fn add_oauth_authorizes_and_adds_the_account_in_one_process() {
    let bench = OauthBench::start().await;
    let (result, url) = bench.login(&[], true).await;
    assert_eq!(result.code, 0, "{}", result.stderr);
    assert!(result.stderr.contains(&url));
    let document = parse_json(&result);
    let account = document["data"]["data"]["id"].as_str().unwrap().to_string();
    assert_eq!(document["data"]["data"]["auth"], "oauth2");
    assert_eq!(document["data"]["data"]["state"]["type"], "connected");
    let access = bench.stored(&account, SecretKind::AccessToken).unwrap();
    let refresh = bench.stored(&account, SecretKind::RefreshToken).unwrap();
    for text in [&result.stdout, &result.stderr] {
        assert!(!text.contains(&access) && !text.contains(&refresh));
    }
}

/// Sans redirection avant `--timeout`, le flux est annulé (13) et rien n'est enregistré
#[tokio::test]
async fn add_oauth_times_out_without_redirection() {
    let bench = OauthBench::start().await;
    let err = SharedBuffer::default();
    let mut deps = bench.deps();
    let args = [
        "account",
        "add-oauth",
        "--address",
        OAUTH_USER,
        "--timeout",
        "1",
    ];
    let limit = Duration::from_secs(30);
    let result = tokio::time::timeout(limit, run_watching(&mut deps, &args, err))
        .await
        .unwrap();
    assert_eq!(result.code, 13, "{}", result.stderr);
    assert!(result.stderr.contains("autorisation non reçue"));
    assert_eq!(bench.secrets.entry_count(), 0);
    let listed = cli_support::run_with(&mut bench.deps(), &["account", "list", "--json"]).await;
    assert!(
        parse_json(&listed)["data"]["data"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

/// Autorisation refusée par l'utilisateur : erreur d'authentification (15), aucun compte ni jeton
#[tokio::test]
async fn add_oauth_denied_by_the_user_registers_nothing() {
    let bench = OauthBench::start().await;
    let (result, _url) = bench.login(&[], false).await;
    assert_eq!(result.code, 15, "{}", result.stderr);
    assert_eq!(bench.secrets.entry_count(), 0);
}

/// Adresse sans provider OAuth2 : refus avant toute attente (10)
#[tokio::test]
async fn add_oauth_for_a_password_only_provider_is_refused() {
    let bench = OauthBench::start().await;
    let mut deps = bench.deps();
    let args = ["account", "add-oauth", "--address", "alice@folders.test"];
    let result = cli_support::run_with(&mut deps, &args).await;
    assert_eq!(result.code, 10, "{}", result.stderr);
    assert_eq!(bench.secrets.entry_count(), 0);
}

/// `--sync-wait` : la première synchro est attendue, le courrier du serveur est lisible ensuite
#[tokio::test]
async fn add_oauth_with_sync_wait_fills_the_mirror() {
    let bench = OauthBench::start().await;
    let raw = b"From: Bob <bob@example.org>\r\nTo: dave@oauth.test\r\nSubject: Bonjour\r\nMessage-ID: <b@x>\r\n\r\nCorps\r\n";
    bench.store.deliver(OAUTH_USER, "INBOX", raw, &[]);
    let (result, _url) = bench.login(&["--sync-wait", "--timeout", "30"], true).await;
    assert_eq!(result.code, 0, "{}", result.stderr);
    let account = parse_json(&result)["data"]["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let boxes = ["mailbox", "list", &account, "--json"];
    let listed = cli_support::run_with(&mut bench.deps(), &boxes).await;
    let data = parse_json(&listed);
    let inbox = data["data"]["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["name"] == "INBOX")
        .unwrap();
    assert_eq!(inbox["total"], 1);
}
