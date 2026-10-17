//! Création et liste des brouillons par la façade.

use mira_core::api::{ComposeRequest, DraftChanges, DraftMode, Request, Response, WriteResponse};
use mira_vocabulary::{Draft, DraftId};

use super::fixture::Fixture;

impl Fixture {
    /// Crée un brouillon dans ce mode, avec l'identité par défaut
    pub async fn create_draft(&self, mode: DraftMode) -> Draft {
        let request = ComposeRequest::CreateDraft {
            account: self.account.clone(),
            identity: None,
            mode,
        };
        self.draft_of(request).await
    }

    /// Brouillons du compte, par la liste
    pub async fn list_drafts(&self) -> Vec<Draft> {
        let request = ComposeRequest::ListDrafts {
            account: self.account.clone(),
        };
        match self.run(Request::Compose(request)).await {
            Response::Write(WriteResponse::Drafts(drafts)) => drafts,
            other => panic!("réponse inattendue : {other:?}"),
        }
    }

    /// Modifie le brouillon par la façade et rend sa nouvelle version
    pub async fn update_draft(&self, draft: &DraftId, changes: DraftChanges) -> Draft {
        let draft = draft.clone();
        self.draft_of(ComposeRequest::UpdateDraft { draft, changes })
            .await
    }

    /// Code de l'erreur d'une requête de rédaction qui doit être refusée
    pub async fn refused_compose(&self, request: ComposeRequest) -> String {
        self.refused(Request::Compose(request)).await.code
    }
}
