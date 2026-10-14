//! Requêtes d'écriture envoyées par la façade, et lectures de contrôle.

use mira_core::api::{QueueRequest, Request, Response, WriteResponse};
use mira_vocabulary::{ActionId, ActionKind, ActionState, MiraError, PendingAction};

use super::builders::apply_request;
use super::fixture::Fixture;

impl Fixture {
    /// Exécute une requête qui doit réussir
    pub async fn run(&self, request: Request) -> Response {
        self.engine().execute(request).await.unwrap()
    }

    /// Exécute une requête qui doit être refusée et rend l'erreur
    pub async fn refused(&self, request: Request) -> MiraError {
        self.engine().execute(request).await.unwrap_err()
    }

    /// Applique l'action à ces messages (clé donnée) et rend les actions mises en file
    pub async fn apply(&self, kind: ActionKind, ids: &[&str], key: &str) -> Vec<ActionId> {
        match self.run(apply_request(kind, ids, key)).await {
            Response::Write(WriteResponse::Actions(actions)) => actions,
            other => panic!("réponse inattendue : {other:?}"),
        }
    }

    /// Actions du compte dans ces états, dans l'ordre de la file
    pub async fn actions(&self, states: &[ActionState]) -> Vec<PendingAction> {
        let request = Request::Queue(QueueRequest::ListActions {
            account: self.account.clone(),
            states: states.to_vec(),
        });
        match self.run(request).await {
            Response::Write(WriteResponse::QueuedActions(actions)) => actions,
            other => panic!("réponse inattendue : {other:?}"),
        }
    }

    /// Code de l'erreur d'une requête qui doit être refusée
    pub async fn refused_code(&self, request: Request) -> String {
        self.refused(request).await.code
    }

    /// Exécute une requête de boîte et rend l'action mise en file
    pub async fn submit_mailbox(&self, request: Request) -> ActionId {
        match self.run(request).await {
            Response::Write(WriteResponse::Action(action)) => action,
            other => panic!("réponse inattendue : {other:?}"),
        }
    }
}
