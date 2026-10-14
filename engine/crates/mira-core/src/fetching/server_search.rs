//! Recherche étendue au serveur (12.4) : critère TEXT sur les boîtes synchronisées, résultat borné.

use mira_vocabulary::{Account, Mailbox, MiraError};

use super::results::{ServerSearchHit, ServerSearchResult};
use super::task::FetchTask;

const MAX_HITS: usize = 200;

/// Boîtes du compte à interroger : celles dont la synchronisation est activée
fn searchable_mailboxes(task: &FetchTask, account: &Account) -> Result<Vec<Mailbox>, MiraError> {
    let all = task.database.read(|tx| tx.list_mailboxes(&account.id))?;
    Ok(all
        .into_iter()
        .filter(|mailbox| mailbox.sync_enabled)
        .collect())
}

/// Associe chaque UID trouvé au message local correspondant, s'il existe
fn hits_of(
    task: &FetchTask,
    mailbox: &Mailbox,
    uids: Vec<u32>,
) -> Result<Vec<ServerSearchHit>, MiraError> {
    task.database.read(|tx| {
        uids.into_iter()
            .map(|uid| {
                let local = tx.appearance_by_uid(&mailbox.id, uid)?;
                Ok(ServerSearchHit {
                    mailbox: mailbox.id.clone(),
                    uid,
                    message: local.map(|appearance| appearance.message),
                })
            })
            .collect()
    })
}

/// Interroge le serveur boîte par boîte, du UID le plus récent au plus ancien, jusqu'à la borne
pub async fn search_server(
    task: &FetchTask,
    account: &Account,
    query: &str,
) -> Result<ServerSearchResult, MiraError> {
    if task.handle.is_cancelled() {
        return Ok(ServerSearchResult::default());
    }
    let mailboxes = searchable_mailboxes(task, account)?;
    let mut client = task.access.connect(account).await?;
    let mut result = ServerSearchResult::default();
    for (index, mailbox) in mailboxes.iter().enumerate() {
        if task.handle.is_cancelled() {
            break;
        }
        task.handle.progress(index as u64, mailboxes.len() as u64);
        client.select(&mailbox.name, None).await?;
        let mut uids = client.search_text(query).await?;
        uids.reverse();
        result.hits.extend(hits_of(task, mailbox, uids)?);
    }
    client.logout().await?;
    result.truncated = result.hits.len() > MAX_HITS;
    result.hits.truncate(MAX_HITS);
    Ok(result)
}
