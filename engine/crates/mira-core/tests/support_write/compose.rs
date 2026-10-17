//! Rédaction et envoi par la façade : brouillons prêts à partir, envoi, boîte d'envoi.

use mira_core::api::{
    ComposeRequest, DraftChanges, DraftMode, OutboxEntry, Request, Response, WriteResponse,
};
use mira_vocabulary::{ActionId, AddressEntry, Draft, DraftId};

use super::builders::send_request;
use super::fixture::Fixture;

/// Destinataire sans nom d'affichage
pub fn recipient(address: &str) -> AddressEntry {
    AddressEntry {
        name: None,
        address: address.to_string(),
    }
}

impl Fixture {
    /// Exécute une requête de rédaction qui rend un brouillon
    pub async fn draft_of(&self, request: ComposeRequest) -> Draft {
        match self.run(Request::Compose(request)).await {
            Response::Write(WriteResponse::Draft(draft)) => *draft,
            other => panic!("réponse inattendue : {other:?}"),
        }
    }

    /// Brouillon neuf, complété pour partir chez bob@example.org
    pub async fn ready_draft(&self) -> Draft {
        let created = ComposeRequest::CreateDraft {
            account: self.account.clone(),
            identity: None,
            mode: DraftMode::New,
        };
        let draft = self.draft_of(created).await;
        let changes = DraftChanges {
            to: Some(vec![recipient("bob@example.org")]),
            subject: Some("Bonjour Bob".to_string()),
            body_text: Some("Corps du message".to_string()),
            ..DraftChanges::default()
        };
        let update = ComposeRequest::UpdateDraft {
            draft: draft.id,
            changes,
        };
        self.draft_of(update).await
    }

    /// Envoie le brouillon avec cette clé et rend l'action
    pub async fn send(&self, draft: &DraftId, key: &str) -> ActionId {
        match self.run(send_request(draft, key)).await {
            Response::Write(WriteResponse::Action(action)) => action,
            other => panic!("réponse inattendue : {other:?}"),
        }
    }

    /// Boîte d'envoi du compte
    pub async fn outbox(&self) -> Vec<OutboxEntry> {
        let request = Request::Compose(ComposeRequest::ListOutbox {
            account: self.account.clone(),
        });
        match self.run(request).await {
            Response::Write(WriteResponse::Outbox(entries)) => entries,
            other => panic!("réponse inattendue : {other:?}"),
        }
    }

    /// Brouillon relu par la façade
    pub async fn get_draft(&self, draft: &DraftId) -> Draft {
        let request = ComposeRequest::GetDraft {
            draft: draft.clone(),
        };
        self.draft_of(request).await
    }
}
