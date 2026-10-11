#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Soumission : vérifications préalables, délai d'annulation, absence de réseau (invariant 28).

mod support;
mod support_actions;
mod support_mirror;

use std::io::ErrorKind;
use std::net::TcpListener;

use mira_core::actions::{UNDO_DELAY_MILLIS, submit};
use mira_core::storage::Database;
use mira_vocabulary::{
    AccountId, ActionKind, ActionState, Security, ServerEndpoint, ServerSettings,
};
use support::accounts::sample_account;
use support_actions::inspect::{action, message, queue_len};
use support_actions::quirks::folders_quirks;
use support_actions::requests::{request, submit_folders};
use support_actions::world::{START_MILLIS, World};

/// Pointe les serveurs du compte acc-1 vers un port local où personne ne doit se connecter
fn point_account_at(db: &Database, port: u16) {
    let endpoint = ServerEndpoint {
        host: "127.0.0.1".to_string(),
        port,
        security: Security::Plaintext,
    };
    let mut account = sample_account("acc-1");
    account.servers = Some(ServerSettings {
        imap: endpoint.clone(),
        smtp: endpoint,
    });
    db.write(|tx| tx.update_account(&account)).unwrap();
}

/// Invariant 28 : la soumission répond sans attendre le réseau ; aucune connexion n'est tentée
#[test]
fn invariant_28_submit_returns_without_network() {
    let world = World::seeded();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    point_account_at(&world.db, listener.local_addr().unwrap().port());
    let id = submit_folders(&world, ActionKind::Trash, &["msg-1"], "key-net");
    assert_eq!(action(&world.db, &id).state, ActionState::Pending);
    let attempt = listener.accept();
    assert!(matches!(attempt, Err(ref e) if e.kind() == ErrorKind::WouldBlock));
}

/// Le délai d'annulation d'une action ordinaire est de 5 s à partir de la soumission
#[test]
fn submitted_action_waits_five_seconds_before_being_ready() {
    let world = World::seeded();
    let id = submit_folders(
        &world,
        ActionKind::MarkRead { read: true },
        &["msg-1"],
        "key-delay",
    );
    let stored = action(&world.db, &id);
    assert_eq!(UNDO_DELAY_MILLIS, 5_000);
    assert_eq!(stored.created_at.as_millis(), START_MILLIS);
    assert_eq!(stored.not_before.as_millis(), START_MILLIS + 5_000);
    let account = AccountId::from_raw("acc-1");
    let ready_early = world
        .db
        .read(|tx| tx.next_ready_action(&account, stored.created_at))
        .unwrap();
    assert!(ready_early.is_none());
    let ready_late = world
        .db
        .read(|tx| tx.next_ready_action(&account, stored.not_before))
        .unwrap();
    assert_eq!(ready_late.map(|a| a.id), Some(id));
}

/// Un message inconnu ou d'un autre compte donne `not_found.message` et ne laisse rien en file
#[test]
fn unknown_or_foreign_message_is_refused_atomically() {
    let world = World::seeded();
    for (target, key) in [("msg-nope", "k1"), ("msg-other", "k2")] {
        let req = request(ActionKind::MarkRead { read: true }, &["msg-1", target], key);
        let error = submit(&world.ctx(), &folders_quirks(), req).unwrap_err();
        assert_eq!(error.code, "not_found.message");
    }
    assert_eq!(queue_len(&world.db), 0);
    assert!(!message(&world.db, "msg-1").flags.seen);
    assert!(world.sink.kinds().is_empty());
}

/// Un compte inconnu, une cible vide et un envoi passé par `submit` sont refusés
#[test]
fn unknown_account_empty_targets_and_send_are_refused() {
    let world = World::seeded();
    let mut req = request(ActionKind::MarkRead { read: true }, &["msg-1"], "k1");
    req.account = AccountId::from_raw("acc-nope");
    let error = submit(&world.ctx(), &folders_quirks(), req).unwrap_err();
    assert_eq!(error.code, "not_found.account");
    let empty = request(ActionKind::MarkRead { read: true }, &[], "k2");
    let error = submit(&world.ctx(), &folders_quirks(), empty).unwrap_err();
    assert_eq!(error.code, "invalid_input.missing_parameter");
    let draft = mira_vocabulary::DraftId::from_raw("d-1");
    let send = request(ActionKind::Send { draft }, &[], "k3");
    let error = submit(&world.ctx(), &folders_quirks(), send).unwrap_err();
    assert_eq!(error.code, "invalid_input.parameter");
    assert_eq!(queue_len(&world.db), 0);
}
