//! Moteur ouvert avec un compte ajouté par la façade et un miroir local garni par un second accès à la base.

use mira_core::api::{MessageRequest, Request, Response};
use mira_core::operations::OperationStatus;
use mira_core::storage::Database;
use mira_vocabulary::{AccountId, AttachmentId, MailboxId, MessageId};

use super::calls::{operation_of, run, wait_finished};
use super::mail;
use super::seed::{add_inbox, add_message};
use crate::support_api::fixture::{self, FOLDERS_USER, Fixture};

/// Ce qu'un test de lecture manipule.
pub struct World {
    pub fixture: Fixture,
    pub account: AccountId,
    pub mailbox: MailboxId,
    pub mirror: Database,
}

impl World {
    /// Adresse de l'utilisateur du faux serveur
    pub fn user(&self) -> &'static str {
        FOLDERS_USER
    }

    /// Dépose le courrier sur le serveur et déclare son en-tête dans le miroir sous l'identifiant `m1`
    pub fn deliver_header_only(&self, raw: &[u8]) -> MessageId {
        let uid = self.fixture.store.deliver(self.user(), "INBOX", raw, &[]);
        add_message(&self.mirror, &self.account, &self.mailbox, "m1", uid, 1_000)
    }

    /// Lance FetchMessageBody et rend l'état final de l'opération
    pub async fn fetch_body(&self, message: &MessageId) -> OperationStatus {
        let request = Request::Message(MessageRequest::FetchMessageBody {
            message: message.clone(),
        });
        let engine = &self.fixture.engine;
        let operation = operation_of(run(engine, request).await.unwrap());
        wait_finished(engine, &operation).await
    }

    /// Garnit la boîte de `count` messages m1…, de plus en plus récents
    pub fn seed_messages(&self, count: u32) {
        for index in 1..=count {
            let id = format!("m{index}");
            let millis = i64::from(index) * 1_000;
            add_message(
                &self.mirror,
                &self.account,
                &self.mailbox,
                &id,
                index,
                millis,
            );
        }
    }
}

/// Monde avec un message à pièce jointe de `size` octets, téléchargé ; rend l'identifiant de la pièce jointe
pub async fn start_world_with_attachment(size: usize) -> (World, AttachmentId) {
    let world = start_world().await;
    let id = world.deliver_header_only(&mail::with_attachment(size));
    assert_eq!(world.fetch_body(&id).await, OperationStatus::Finished);
    let list = Request::Message(MessageRequest::ListAttachments { message: id });
    let Response::Attachments(found) = run(&world.fixture.engine, list).await.unwrap() else {
        panic!("réponse inattendue");
    };
    let attachment = found[0].id.clone();
    (world, attachment)
}

/// Démarre le faux serveur, ajoute le compte par la façade et crée la boîte INBOX locale
pub async fn start_world() -> World {
    let fixture = fixture::start().await;
    let account = fixture.add_folders_account("key-read").await;
    let path = fixture.home.path().join("profiles/default/mira.sqlite3");
    let mirror = Database::open(&path).unwrap();
    let mailbox = add_inbox(&mirror, &account);
    World {
        fixture,
        account,
        mailbox,
        mirror,
    }
}
