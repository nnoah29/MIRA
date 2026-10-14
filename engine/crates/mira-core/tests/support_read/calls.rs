//! Appels de la façade pour les tests de lecture : exécution, pages, identifiants d'opération, journal des sorties.

pub use super::operations::wait_finished;

use mira_core::api::{Engine, MessageRequest, Request, Response};
use mira_core::reading::ListQuery;
use mira_vocabulary::{
    Cursor, MailboxId, MessageSummary, MiraError, OperationId, Page, PageRequest, ViewRef,
};

/// Exécute une requête et rend sa réponse ou son erreur
pub async fn run(engine: &Engine, request: Request) -> Result<Response, MiraError> {
    engine.execute(request).await
}

/// Une page de messages de la boîte, avec le curseur de reprise éventuel
pub async fn messages_page(
    engine: &Engine,
    mailbox: &MailboxId,
    limit: u32,
    cursor: Option<Cursor>,
) -> Result<Response, MiraError> {
    run(
        engine,
        Request::Message(MessageRequest::ListMessages(ListQuery {
            view: ViewRef::Mailbox {
                mailbox: mailbox.clone(),
            },
            filters: Default::default(),
            page: PageRequest { cursor, limit },
        })),
    )
    .await
}

/// Extrait la page d'une réponse de liste de messages
pub fn page_of(response: Response) -> Page<MessageSummary> {
    match response {
        Response::Messages(page) => page,
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Identifiant de l'opération démarrée par une réponse
pub fn operation_of(response: Response) -> OperationId {
    match response {
        Response::Operation(info) => info.id,
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Ajoute à `seen` le JSON de la réponse ou de l'erreur, comme un client pourrait le voir
pub fn record(seen: &mut String, result: &Result<Response, MiraError>) {
    let text = match result {
        Ok(response) => serde_json::to_string(response),
        Err(error) => serde_json::to_string(error),
    };
    seen.push_str(&text.unwrap());
}
