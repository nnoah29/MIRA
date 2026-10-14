#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! S10 par le CLI : jeton expiré, rafraîchissement silencieux ou compte à réauthentifier, file conservée (7.7, inv. 15, 17).

#[allow(dead_code)]
mod cli_support;

use cli_support::sb_events::{events_of_type, first_seq_of_type};
use cli_support::sb_leaks::assert_outputs_clean;
use cli_support::sb_oauth::OAUTH_USER;
use cli_support::sb_world::SbWorld;
use cli_support::{CliResult, account_id_of};
use mira_assembly::mira_vocabulary::SecretKind;
use mira_testkit::imap::CapabilitySet;

/// Compte OAuth2 ajouté avec sa première synchro ; rend le monde, le compte, et les messages Alpha et Beta.
async fn oauth_world() -> (SbWorld, String, String, String) {
    let world = SbWorld::start(&[], CapabilitySet::full()).await;
    world.deliver_subject(OAUTH_USER, "INBOX", "Alpha");
    world.deliver_subject(OAUTH_USER, "INBOX", "Beta");
    let login = world.oauth_login(&["--sync-wait", "--timeout", "30"]).await;
    assert_eq!(login.code, 0, "{}", login.stderr);
    let account = account_id_of(&login);
    let inbox = world.mailbox_named(&account, "INBOX").await;
    let alpha = world.message_id_of(&inbox, "Alpha").await;
    let beta = world.message_id_of(&inbox, "Beta").await;
    (world, account, alpha, beta)
}

/// Attend au plus 3 s qu'une action de plus soit rejouée, l'horloge avançant seule ; rend le résultat.
async fn wait_three_seconds(world: &SbWorld, message: &str) -> CliResult {
    let pump = world.pump_clock();
    let line = [
        "action",
        "apply",
        "flag",
        "--message",
        message,
        "--wait",
        "--timeout",
        "3",
    ];
    let result = world.run(&line).await;
    pump.abort();
    result
}

/// Rafraîchissement impossible : le compte demande une authentification, les actions restent en file sans échouer
#[tokio::test]
async fn scenario_s10_revoked_token_keeps_actions_queued_via_cli() {
    let (world, account, alpha, beta) = oauth_world().await;
    world.oauth.revoke_refresh_tokens(OAUTH_USER);
    world.oauth.expire_access_tokens(OAUTH_USER);
    let action = world.apply_pending("mark-read", &alpha).await;
    let waited = wait_three_seconds(&world, &beta).await;
    assert_eq!(waited.code, 13, "{}", waited.stderr);
    let listed = world.run_ok(&["account", "list"]).await;
    assert_eq!(listed["data"][0]["state"]["type"], "auth_required");
    let actions = world.queued_actions(&account).await;
    assert_eq!(actions.len(), 2);
    assert!(actions.iter().all(|queued| queued["state"] == "pending"));
    assert_eq!(actions[0]["id"], action.as_str());
    let events = world.events().await;
    let changes = events_of_type(&events, "account_state_changed");
    assert_eq!(changes.last().unwrap()["state"]["type"], "auth_required");
    assert!(first_seq_of_type(&events, "action_failed").is_none());
    assert!(!world.server_seen(OAUTH_USER, "INBOX", 1));
}

/// Jeton d'accès expiré mais rafraîchissable : l'opération continue sans que le compte quitte « connecté »
#[tokio::test]
async fn scenario_s10_expired_access_token_is_refreshed_silently_via_cli() {
    let (world, account, alpha, beta) = oauth_world().await;
    let before = world
        .stored_token(&account, SecretKind::AccessToken)
        .unwrap();
    let requests = world.oauth.token_requests();
    world.oauth.expire_access_tokens(OAUTH_USER);
    world.apply_pending("mark-read", &alpha).await;
    world.replay_queue_through(&beta).await;
    assert!(world.server_seen(OAUTH_USER, "INBOX", 1));
    assert!(world.oauth.token_requests() > requests);
    let after = world
        .stored_token(&account, SecretKind::AccessToken)
        .unwrap();
    assert_ne!(before, after);
    let listed = world.run_ok(&["account", "list"]).await;
    assert_eq!(listed["data"][0]["state"]["type"], "connected");
    let events = world.events().await;
    let changes = events_of_type(&events, "account_state_changed");
    assert!(
        changes
            .iter()
            .all(|c| c["state"]["type"] != "auth_required")
    );
    assert!(!world.profile_contains(&before) && !world.profile_contains(&after));
}

/// Mot de passe refusé : action en file, compte à réauthentifier, la file reprend après le nouveau mot de passe
#[tokio::test]
async fn scenario_s10_new_password_resumes_the_queue_via_cli() {
    let user = "alice@folders.test";
    let world = SbWorld::start(&[user], CapabilitySet::full()).await;
    world.deliver_subject(user, "INBOX", "Alpha");
    world.deliver_subject(user, "INBOX", "Beta");
    let account = world.add_account(user).await;
    world.run_sync_ok(&["sync", "start"]).await;
    let inbox = world.mailbox_named(&account, "INBOX").await;
    let alpha = world.message_id_of(&inbox, "Alpha").await;
    let beta = world.message_id_of(&inbox, "Beta").await;
    world.credentials.add_password(user, "nouveau-mdp-4455");
    let action = world.apply_pending("mark-read", &alpha).await;
    assert_eq!(wait_three_seconds(&world, &beta).await.code, 13);
    let listed = world.run_ok(&["account", "list"]).await;
    assert_eq!(listed["data"][0]["state"]["type"], "auth_required");
    let store = ["auth", "app-password", &account, "--password-stdin"];
    let stored = world.run_stdin("nouveau-mdp-4455\n", &store).await;
    assert_eq!(stored.code, 0, "{}", stored.stderr);
    world.replay_queue_through(&beta).await;
    assert!(world.server_seen(user, "INBOX", 1));
    let actions = world.queued_actions(&account).await;
    assert_eq!(actions[0]["id"], action.as_str());
    assert!(actions.iter().all(|queued| queued["state"] == "succeeded"));
    assert_outputs_clean(&[&stored], "nouveau-mdp-4455");
    assert!(!world.profile_contains("nouveau-mdp-4455"));
}

/// `auth reauthenticate` ouvre un nouveau flux : l'adresse d'autorisation est rendue, la file reste intacte
#[tokio::test]
async fn scenario_s10_reauthenticate_prints_the_authorization_url_via_cli() {
    let (world, account, alpha, beta) = oauth_world().await;
    world.oauth.revoke_refresh_tokens(OAUTH_USER);
    world.oauth.expire_access_tokens(OAUTH_USER);
    world.apply_pending("mark-read", &alpha).await;
    wait_three_seconds(&world, &beta).await;
    let access = world
        .stored_token(&account, SecretKind::AccessToken)
        .unwrap();
    let refresh = world
        .stored_token(&account, SecretKind::RefreshToken)
        .unwrap();
    let flow = world.run(&["auth", "reauthenticate", &account]).await;
    assert_eq!(flow.code, 0, "{}", flow.stderr);
    assert!(flow.stdout.contains(&world.oauth.authorize_url()));
    let queued = world.queued_actions(&account).await;
    assert!(queued.iter().all(|action| action["state"] == "pending"));
    assert_outputs_clean(&[&flow], &access);
    assert_outputs_clean(&[&flow], &refresh);
    assert!(!world.profile_contains(&refresh));
}
