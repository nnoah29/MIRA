//! Courrier reçu de test pour composer des réponses et des transferts.

use mira_vocabulary::{AddressEntry, MailboxId, MessageId, MessageSummary, Timestamp};

use super::world::World;
use crate::support_mirror::fixtures::{appearance, sample_message};

/// Adresse de test sans nom d'affichage
pub fn entry(address: &str) -> AddressEntry {
    AddressEntry {
        name: None,
        address: address.to_string(),
    }
}

/// Adresses d'une liste de destinataires, en minuscules
pub fn addresses(list: &[AddressEntry]) -> Vec<String> {
    list.iter().map(|e| e.address.to_lowercase()).collect()
}

/// Message reçu par acc-1 : `bob@example.org` écrit à `main@example.org` et `carol@example.org`, objet et en-têtes de fil choisis
pub fn incoming(id: &str, subject: &str) -> MessageSummary {
    let mut summary = sample_message(id, "acc-1");
    summary.subject = subject.to_string();
    summary.from = vec![entry("bob@example.org")];
    summary.to = vec![entry("main@example.org"), entry("carol@example.org")];
    summary.cc = vec![entry("Dave@Example.org")];
    summary.message_id_header = Some(format!("<{id}@mail.example.org>"));
    summary.references = vec!["<root@mail.example.org>".to_string()];
    summary
}

/// Insère le message dans la réception d'acc-1 et rend son identifiant
pub fn store(world: &World, summary: &MessageSummary) -> MessageId {
    world
        .db
        .write(|tx| {
            tx.insert_message(summary, Timestamp::from_millis(9_000))?;
            let inbox = MailboxId::from_raw("box-in");
            let uid = 50 + u32::try_from(tx.uids_in_mailbox(&inbox)?.len()).unwrap();
            tx.add_appearance(&appearance(summary.id.as_str(), "box-in", uid))
        })
        .unwrap();
    summary.id.clone()
}
