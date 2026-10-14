//! Lectures du miroir par la façade après une synchronisation : boîtes, messages, identifiants, objets.

use mira_core::api::{Engine, MailboxRequest, MessageRequest, Request, Response};
use mira_core::reading::ListQuery;
use mira_vocabulary::{AccountId, Mailbox, MessageId, MessageSummary, PageRequest, ViewRef};

/// Boîtes du miroir d'un compte
pub async fn mailboxes_of(engine: &Engine, account: &AccountId) -> Vec<Mailbox> {
    let request = Request::Mailbox(MailboxRequest::ListMailboxes {
        account: account.clone(),
    });
    match engine.execute(request).await.unwrap() {
        Response::Mailboxes(list) => list,
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Boîte du miroir portant ce nom
pub async fn mailbox_named(engine: &Engine, account: &AccountId, name: &str) -> Mailbox {
    let found = mailboxes_of(engine, account).await;
    found.into_iter().find(|m| m.name == name).unwrap()
}

/// Messages de la boîte, du plus récent au plus ancien
pub async fn summaries_in(engine: &Engine, account: &AccountId, name: &str) -> Vec<MessageSummary> {
    let mailbox = mailbox_named(engine, account, name).await;
    let view = ViewRef::Mailbox {
        mailbox: mailbox.id,
    };
    let page = PageRequest {
        cursor: None,
        limit: 200,
    };
    let query = ListQuery {
        view,
        filters: Default::default(),
        page,
    };
    let request = Request::Message(MessageRequest::ListMessages(query));
    match engine.execute(request).await.unwrap() {
        Response::Messages(page) => page.items,
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Objets des messages de la boîte
pub async fn subjects_in(engine: &Engine, account: &AccountId, name: &str) -> Vec<String> {
    let summaries = summaries_in(engine, account, name).await;
    summaries.into_iter().map(|m| m.subject).collect()
}

/// Identifiants internes des messages de la boîte, triés
pub async fn ids_in(engine: &Engine, account: &AccountId, name: &str) -> Vec<MessageId> {
    let summaries = summaries_in(engine, account, name).await;
    let mut ids: Vec<MessageId> = summaries.into_iter().map(|m| m.id).collect();
    ids.sort();
    ids
}

/// Attend (au plus 15 s) que la boîte contienne exactement `count` messages et rend leurs objets
pub async fn wait_for_subjects(
    engine: &Engine,
    account: &AccountId,
    name: &str,
    count: usize,
) -> Vec<String> {
    let wait = async {
        loop {
            let found = subjects_in(engine, account, name).await;
            if found.len() == count {
                return found;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    };
    tokio::time::timeout(std::time::Duration::from_secs(15), wait)
        .await
        .expect("messages attendus jamais synchronisés")
}
