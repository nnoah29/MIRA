#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Invariant 27 : aucune lecture ordinaire ne passe par le réseau.

mod support;
mod support_mirror;
mod support_reading;

use std::io::ErrorKind;
use std::net::TcpListener;
use std::path::Path;

use mira_core::reading::{
    ListQuery, SearchFilters, get_body, get_message, get_raw_source, list_attachments,
    list_messages, mailboxes_with_counters, search,
};
use mira_core::storage::Database;
use mira_core::threads::{get_thread, list_threads};
use mira_vocabulary::{
    AccountId, MailboxId, MessageId, PageRequest, Security, ServerEndpoint, ServerSettings,
    StandardView, ThreadId, ViewRef,
};
use support::accounts::sample_account;
use support_reading::bodies::database_with_bodies;
use support_reading::queries::profile;

/// Remplace les serveurs du compte acc-1 par une adresse locale où un faux serveur écoute
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

/// Liste la première page de la boîte et de chaque vue standard
fn list_every_view(db: &Database) {
    let views = [
        ViewRef::Mailbox {
            mailbox: MailboxId::from_raw("box-1"),
        },
        ViewRef::Standard {
            view: StandardView::Unified,
        },
        ViewRef::Standard {
            view: StandardView::Unread,
        },
        ViewRef::Standard {
            view: StandardView::Flagged,
        },
    ];
    for view in views {
        list_messages(db, &profile(), &ListQuery::of(view)).unwrap();
    }
}

/// Lance toutes les lectures locales du jalon, y compris celles qui échouent faute de corps
fn read_everything(db: &Database) {
    let id = MessageId::from_raw("full");
    list_every_view(db);
    get_message(db, &id).unwrap();
    get_body(db, &id).unwrap();
    get_raw_source(db, &MessageId::from_raw("plain")).unwrap_err();
    list_attachments(db, &id).unwrap();
    search(
        db,
        &profile(),
        "Sujet",
        &SearchFilters::default(),
        &PageRequest::default(),
    )
    .unwrap();
    list_threads(db, &MailboxId::from_raw("box-1"), &PageRequest::default()).unwrap();
    get_thread(db, &ThreadId::from_raw("nope")).unwrap_err();
    mailboxes_with_counters(db, &AccountId::from_raw("acc-1")).unwrap();
}

/// Invariant 27 : pendant une batterie de lectures, le serveur configuré du compte ne reçoit aucune connexion
#[test]
fn invariant_27_reads_never_touch_network() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let db = database_with_bodies();
    point_account_at(&db, listener.local_addr().unwrap().port());
    read_everything(&db);
    let accepted = listener.accept();
    assert_eq!(accepted.unwrap_err().kind(), ErrorKind::WouldBlock);
}

/// Les modules de lecture et de fils ne mentionnent aucune API réseau ni module réseau du cœur
#[test]
fn reading_and_threads_sources_use_no_network_api() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let forbidden = [
        "std::net",
        "tokio::net",
        "TcpStream",
        "reqwest",
        "lettre",
        "async_imap",
        "crate::imap",
        "crate::smtp",
        "crate::auth",
    ];
    for module in ["reading", "threads"] {
        for entry in std::fs::read_dir(root.join(module)).unwrap() {
            let path = entry.unwrap().path();
            let text = std::fs::read_to_string(&path).unwrap();
            for word in forbidden {
                assert!(!text.contains(word), "{} mentionne {word}", path.display());
            }
        }
    }
}
