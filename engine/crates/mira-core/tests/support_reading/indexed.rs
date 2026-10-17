//! Messages indexés pour la recherche : sujets, corps, expéditeurs, destinataires, drapeaux.

use mira_core::storage::Database;
use mira_vocabulary::{AddressEntry, MessageId};

use super::views::{database_with_many_boxes, put_custom, sender};
use crate::support_mirror::fixtures::sample_message;

/// Ligne de données : id, compte, boîte, date, nom, adresse, sujet, corps
type Row = (
    &'static str,
    &'static str,
    &'static str,
    i64,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
);

const ROWS: [Row; 4] = [
    (
        "s1",
        "acc-1",
        "box-1",
        1_000,
        "Bob",
        "bob@example.org",
        "Facture avril",
        "facture du plombier",
    ),
    (
        "s2",
        "acc-1",
        "label-1",
        2_000,
        "Carole",
        "carole@example.net",
        "Facture mai",
        "facture électricité",
    ),
    (
        "s3",
        "acc-2",
        "box-2",
        3_000,
        "Bob",
        "bob@example.org",
        "Réunion",
        "ordre du jour facture",
    ),
    (
        "s4",
        "acc-1",
        "box-1",
        4_000,
        "Dan",
        "dan@example.org",
        "Vacances",
        "plage et soleil",
    ),
];

/// Base avec quatre messages indexés : s1 (acc-1, box-1, pièce jointe), s2 (acc-1, label-1, suivi, copie à dave), s3 (acc-2), s4 (acc-1, box-1)
pub fn database_with_indexed_messages() -> Database {
    let db = database_with_many_boxes();
    for (id, account, mailbox, at, name, address, subject, body) in ROWS {
        let mut message = sample_message(id, account);
        message.from = sender(name, address);
        message.subject = subject.to_string();
        message.has_attachments = id == "s1";
        message.flags.flagged = id == "s2";
        message.flags.seen = id != "s2" && id != "s3";
        message.cc = match id {
            "s2" => sender("Dave", "dave@example.com"),
            _ => Vec::<AddressEntry>::new(),
        };
        put_custom(&db, &message, at, &[mailbox]);
        let who = format!("{name} {address}");
        db.write(|tx| tx.index_message(&MessageId::from_raw(id), subject, &who, body))
            .unwrap();
    }
    db
}
