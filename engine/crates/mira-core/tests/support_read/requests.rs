//! Requêtes de la façade préparées pour les tests de lecture.

use std::path::Path;

use mira_core::api::{
    AttachmentTarget, EventRequest, MailboxRequest, MaintenanceRequest, MessageRequest, Request,
    SearchRequest, ThreadRequest,
};
use mira_vocabulary::{AttachmentId, EventFilter, MailboxId, MessageId, OperationId, PageRequest};

use super::calls::{operation_of, run};
use super::world::World;

impl World {
    /// Lance SaveAttachment vers `destination` et rend l'identifiant de l'opération
    pub async fn start_save(&self, attachment: &AttachmentId, destination: &Path) -> OperationId {
        let request = Request::Message(MessageRequest::SaveAttachment(AttachmentTarget {
            attachment: attachment.clone(),
            destination: destination.to_string_lossy().to_string(),
        }));
        operation_of(run(&self.fixture.engine, request).await.unwrap())
    }

    /// Requête qui choisit ces boîtes comme boîtes synchronisées du compte
    pub fn select_synced(&self, mailboxes: Vec<MailboxId>) -> Request {
        Request::Mailbox(MailboxRequest::SetSyncedMailboxes {
            account: self.account.clone(),
            mailboxes,
        })
    }
}

/// Lectures locales d'un message : en-têtes, corps, source, pièces jointes, HTML sûr
pub fn message_reads(id: &MessageId) -> Vec<Request> {
    let message = id.clone();
    vec![
        Request::Message(MessageRequest::GetMessage {
            message: message.clone(),
        }),
        Request::Message(MessageRequest::GetMessageBody {
            message: message.clone(),
        }),
        Request::Message(MessageRequest::GetRawSource {
            message: message.clone(),
        }),
        Request::Message(MessageRequest::ListAttachments {
            message: message.clone(),
        }),
        Request::Message(MessageRequest::GetSafeHtml { message }),
    ]
}

/// Lectures locales de la boîte : arbre des boîtes, liste des fils, recherche plein texte
pub fn mailbox_reads(world: &World, query: &str) -> Vec<Request> {
    vec![
        Request::Mailbox(MailboxRequest::ListMailboxes {
            account: world.account.clone(),
        }),
        Request::Thread(ThreadRequest::ListThreads {
            mailbox: world.mailbox.clone(),
            page: PageRequest::default(),
        }),
        Request::Search(SearchRequest::Search {
            query: query.to_string(),
            filters: Default::default(),
            page: PageRequest::default(),
        }),
    ]
}

/// Exports de maintenance vers deux fichiers du dossier donné, puis lecture de tous les événements
pub fn exports_and_events(directory: &Path) -> Vec<Request> {
    let target = |name: &str| directory.join(name).to_string_lossy().to_string();
    vec![
        Request::Maintenance(MaintenanceRequest::ExportDiagnostic {
            destination: target("d.json"),
        }),
        Request::Maintenance(MaintenanceRequest::ExportProfile {
            destination: target("p.json"),
        }),
        Request::Event(EventRequest::GetEvents {
            filter: EventFilter::default(),
            limit: 1000,
        }),
    ]
}
