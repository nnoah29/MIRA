#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! S14 par le CLI : suppression d'un compte, données et secrets effacés, autres comptes et serveur intacts (inv. 15, 16).

#[allow(dead_code)]
mod cli_support;

use cli_support::json_views::page_ids;
use cli_support::sb_events::events_of_type;
use cli_support::sb_leaks::assert_outputs_clean;
use cli_support::sb_parts::AccountParts;
use cli_support::sb_world::{SB_PASSWORD, SbWorld};
use mira_assembly::mira_vocabulary::SecretKind;
use mira_testkit::imap::CapabilitySet;

const DOOMED: &str = "alice@folders.test";
const KEPT: &str = "carol@folders.test";

/// Deux comptes synchronisés, chacun avec un brouillon et une action en attente ; rend le monde et leurs objets.
async fn two_accounts() -> (SbWorld, AccountParts, AccountParts) {
    let world = SbWorld::start(&[DOOMED, KEPT], CapabilitySet::full()).await;
    world.deliver_subject(DOOMED, "INBOX", "Zeta-condamne");
    world.deliver_subject(DOOMED, "INBOX", "Zeta-deux");
    world.deliver_subject(KEPT, "INBOX", "Kappa-conserve");
    let doomed = world.add_account(DOOMED).await;
    let kept = world.add_account(KEPT).await;
    world.run_sync_ok(&["sync", "start"]).await;
    let doomed_parts = world.parts_of(&doomed, "Zeta-condamne").await;
    let kept_parts = world.parts_of(&kept, "Kappa-conserve").await;
    (world, doomed_parts, kept_parts)
}

/// Le compte supprimé disparaît : liste, miroir, brouillons, file, secret du trousseau ; événement émis
#[tokio::test]
async fn scenario_s14_delete_erases_local_data_and_secrets_via_cli() {
    let (world, doomed, kept) = two_accounts().await;
    assert_eq!(world.secret_count(), 2);
    let deleted = world.run(&["account", "delete", &doomed.account]).await;
    assert_eq!(deleted.code, 0, "{}", deleted.stderr);
    let listed = world.run_ok(&["account", "list"]).await;
    assert_eq!(listed["data"].as_array().unwrap().len(), 1);
    assert_eq!(listed["data"][0]["id"], kept.account.as_str());
    assert!(
        world
            .stored_token(&doomed.account, SecretKind::Password)
            .is_none()
    );
    assert_eq!(world.secret_count(), 1);
    world.assert_parts_gone(&doomed).await;
    let events = world.events().await;
    let removed = events_of_type(&events, "account_deleted");
    assert_eq!(removed.len(), 1);
    assert_eq!(removed[0]["account"], doomed.account.as_str());
    let found = world.run_ok(&["search", "local", "Zeta-condamne"]).await;
    assert!(page_ids(&found).is_empty());
    let kept_found = world.run_ok(&["search", "local", "Kappa-conserve"]).await;
    assert_eq!(page_ids(&kept_found), [kept.message.as_str()]);
}

/// Les autres comptes sont intacts et le serveur ne reçoit rien : ni commande, ni connexion, ni message
#[tokio::test]
async fn scenario_s14_other_accounts_and_the_server_are_untouched_via_cli() {
    let (world, doomed, kept) = two_accounts().await;
    let commands = world.store.command_log();
    let connections = world.imap.connection_count();
    let snapshots = (world.server_snapshot(DOOMED), world.server_snapshot(KEPT));
    world.run_ok(&["account", "delete", &doomed.account]).await;
    assert_eq!(world.store.command_log(), commands);
    assert_eq!(world.imap.connection_count(), connections);
    assert_eq!(world.server_snapshot(DOOMED), snapshots.0);
    assert_eq!(world.server_snapshot(KEPT), snapshots.1);
    assert!(world.smtp.received().is_empty());
    assert_eq!(world.subjects_of(&kept.inbox).await, ["Kappa-conserve"]);
    let draft = world.run_ok(&["draft", "get", &kept.draft]).await;
    assert_eq!(draft["data"]["data"]["account"], kept.account.as_str());
    let queued = world.queued_actions(&kept.account).await;
    assert_eq!(queued[0]["id"], kept.action.as_str());
    assert_eq!(queued[0]["state"], "pending");
    assert!(
        world
            .stored_token(&kept.account, SecretKind::Password)
            .is_some()
    );
}

/// Après la suppression, le compte reste absent à la commande suivante et le compte gardé continue de se synchroniser
#[tokio::test]
async fn scenario_s14_deletion_is_final_and_the_kept_account_keeps_syncing_via_cli() {
    let (world, doomed, kept) = two_accounts().await;
    let deleted = world.run(&["account", "delete", &doomed.account]).await;
    let again = world.run(&["account", "delete", &doomed.account]).await;
    assert_eq!(again.code, 11, "{}", again.stderr);
    world.deliver_subject(KEPT, "INBOX", "Kappa-nouveau");
    world.run_sync_ok(&["sync", "start"]).await;
    let subjects = world.subjects_of(&kept.inbox).await;
    assert_eq!(subjects, ["Kappa-conserve", "Kappa-nouveau"]);
    let status = world.run_ok(&["sync", "status"]).await;
    assert_eq!(status["data"]["data"].as_array().unwrap().len(), 1);
    let events = world.run(&["events", "--json"]).await;
    assert_outputs_clean(&[&deleted, &again, &events], SB_PASSWORD);
    assert!(!world.profile_contains(SB_PASSWORD));
}

/// Aucun octet du profil (base, journal, verrou) ne garde le sujet ni l'adresse du compte supprimé
#[tokio::test]
async fn scenario_s14_no_trace_of_the_deleted_account_remains_in_profile_files_via_cli() {
    let (world, doomed, _kept) = two_accounts().await;
    assert!(world.profile_contains("Zeta-condamne"));
    assert!(world.profile_contains(DOOMED));
    let deleted = world.run(&["account", "delete", &doomed.account]).await;
    assert_eq!(deleted.code, 0, "{}", deleted.stderr);
    assert!(!world.profile_contains("Zeta-condamne"), "sujet gardé");
    assert!(!world.profile_contains("Zeta-deux"), "second sujet gardé");
    assert!(!world.profile_contains(DOOMED), "adresse gardée");
}
