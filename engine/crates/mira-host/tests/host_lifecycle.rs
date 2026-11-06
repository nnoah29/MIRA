#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Cycle de vie de l'hôte : fermé, ouvert deux fois, partagé, fermé pendant un appel.

mod support;

use mira_host::Host;
use serde_json::{Value, json};

const LIST_ACCOUNTS: &str = r#"{"domain":"account","request":{"op":"list_accounts"}}"#;

/// Décode une enveloppe JSON.
fn decode(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

/// Un hôte jamais ouvert répond par une enveloppe d'échec au message fixe, sans panique.
#[tokio::test(flavor = "multi_thread")]
async fn a_host_never_opened_answers_with_a_fixed_failure() {
    let host = Host::new();
    let failed = decode(&host.execute("{}").await);
    assert_eq!(failed["ok"], false);
    assert_eq!(failed["error"]["message"], "moteur non ouvert");
    assert_eq!(decode(&host.close().await)["ok"], true);
}

/// Deux hôtes d'un même processus ne partagent rien.
#[tokio::test(flavor = "multi_thread")]
async fn two_hosts_do_not_share_their_engine() {
    let home = tempfile::tempdir().unwrap();
    let (a, b) = (Host::new(), Host::new());
    assert_eq!(
        decode(&a.open(support::config(home.path(), "a"))).get("ok"),
        Some(&json!(true))
    );
    assert!(b.engine().is_none());
    let failed = decode(&b.execute(LIST_ACCOUNTS).await);
    assert_eq!(failed["error"]["message"], "moteur non ouvert");
    assert_eq!(decode(&a.execute(LIST_ACCOUNTS).await)["ok"], true);
}

/// Le verrou de profil refuse la seconde ouverture ; le premier moteur continue de répondre.
#[tokio::test(flavor = "multi_thread")]
async fn a_second_open_of_the_same_profile_is_refused_and_the_first_keeps_working() {
    let home = tempfile::tempdir().unwrap();
    let (first, second) = (Host::new(), Host::new());
    assert_eq!(
        decode(&first.open(support::config(home.path(), "p")))["ok"],
        true
    );
    assert_eq!(
        decode(&second.open(support::config(home.path(), "p")))["ok"],
        false
    );
    assert_eq!(decode(&first.execute(LIST_ACCOUNTS).await)["ok"], true);
}

/// Fermer pendant qu'un appel tient le moteur échoue, puis l'hôte est vide.
#[tokio::test(flavor = "multi_thread")]
async fn close_while_the_engine_is_in_use_fails_then_leaves_the_host_empty() {
    let home = tempfile::tempdir().unwrap();
    let host = Host::new();
    assert_eq!(
        decode(&host.open(support::config(home.path(), "c")))["ok"],
        true
    );
    let held = host.engine().unwrap();
    let closing = decode(&host.close().await);
    assert_eq!(closing["ok"], false);
    assert_eq!(closing["error"]["message"], "moteur encore utilisé");
    drop(held);
    let after = decode(&host.execute(LIST_ACCOUNTS).await);
    assert_eq!(after["error"]["message"], "moteur non ouvert");
}
