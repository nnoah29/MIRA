#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Authentification du faux serveur IMAP : LOGIN, PLAIN, XOAUTH2, CAPABILITY.

mod imap_support;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use imap_support::client::RawClient;
use imap_support::fixtures::start;
use imap_support::within;
use mira_testkit::imap::CapabilitySet;

/// Un mot de passe connu du registre ouvre la session
#[tokio::test]
async fn login_accepts_valid_password() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        let mut client = RawClient::connect(&env.server.address()).await;
        assert!(client.greeting.contains("AUTH=PLAIN AUTH=XOAUTH2"));
        let response = client.command("LOGIN alice secret").await;
        assert!(response.is_ok(), "{response:?}");
        assert!(response.status.contains("CAPABILITY"));
    })
    .await;
}

/// Un mauvais mot de passe est refusé et ne figure jamais dans le journal
#[tokio::test]
async fn login_refuses_wrong_password_and_hides_it_in_the_log() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        let mut client = RawClient::connect(&env.server.address()).await;
        let response = client.command("LOGIN alice \"wrong pass\"").await;
        assert!(
            response.status.contains("NO [AUTHENTICATIONFAILED]"),
            "{response:?}"
        );
        let log = env.store.command_log();
        assert_eq!(log, vec!["LOGIN alice ***".to_string()]);
    })
    .await;
}

/// XOAUTH2 accepte un jeton d'accès valide
#[tokio::test]
async fn authenticate_xoauth2_accepts_valid_token() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        let mut client = RawClient::connect(&env.server.address()).await;
        let payload = STANDARD.encode("user=alice\u{1}auth=Bearer tok-1\u{1}\u{1}");
        let response = client
            .command(&format!("AUTHENTICATE XOAUTH2 {payload}"))
            .await;
        assert!(response.is_ok(), "{response:?}");
        assert!(client.command("LIST \"\" \"*\"").await.has("INBOX"));
    })
    .await;
}

/// XOAUTH2 refuse un jeton révoqué
#[tokio::test]
async fn authenticate_xoauth2_refuses_revoked_token() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.credentials.revoke_access_token("tok-1");
        let mut client = RawClient::connect(&env.server.address()).await;
        let payload = STANDARD.encode("user=alice\u{1}auth=Bearer tok-1\u{1}\u{1}");
        let response = client
            .command(&format!("AUTHENTICATE XOAUTH2 {payload}"))
            .await;
        assert!(
            response.status.contains("NO [AUTHENTICATIONFAILED]"),
            "{response:?}"
        );
    })
    .await;
}

/// PLAIN sans réponse initiale passe par une ligne de continuation
#[tokio::test]
async fn authenticate_plain_uses_continuation_line() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        let mut client = RawClient::connect(&env.server.address()).await;
        client.send_raw("p1 AUTHENTICATE PLAIN\r\n").await;
        assert_eq!(client.read_line().await.unwrap(), "+ ");
        client
            .send_raw(&format!("{}\r\n", STANDARD.encode("\0alice\0secret")))
            .await;
        assert!(client.read_response("p1").await.is_ok());
    })
    .await;
}

/// CAPABILITY reflète l'ensemble d'extensions choisi
#[tokio::test]
async fn capability_reflects_the_chosen_set() {
    within(async {
        let minimal = start(CapabilitySet::minimal()).await;
        let mut client = RawClient::connect(&minimal.server.address()).await;
        let line = client.command("CAPABILITY").await.line("CAPABILITY");
        assert_eq!(line, "* CAPABILITY IMAP4rev1 AUTH=PLAIN AUTH=XOAUTH2");
        let full = start(CapabilitySet::full()).await;
        let mut client = RawClient::connect(&full.server.address()).await;
        assert!(client.command("CAPABILITY").await.has("QRESYNC"));
    })
    .await;
}
