//! Retrouver sur le serveur les messages visés : par UID s'il est sûr, sinon par `Message-ID` (inv. 18).

use std::collections::BTreeMap;

use mira_vocabulary::error::codes::server;
use mira_vocabulary::{MessageId, MiraError};

use crate::actions::replay::job::ServerMessage;
use crate::imap::{ImapClient, SelectedMailbox};

/// Quelles apparitions d'un message l'action vise.
#[derive(Debug, Clone)]
pub(super) enum Scope {
    All,
    Except(String),
    Only(String),
}

/// Un message cherché dans une boîte : UID connu (0 si inconnu), validité d'UID et `Message-ID`.
#[derive(Debug, Clone)]
pub(super) struct Wanted {
    pub message: MessageId,
    pub uid: u32,
    pub validity: Option<u32>,
    pub header: Option<String>,
}

/// Messages retrouvés (avec leur UID actuel) et messages absents de la boîte.
#[derive(Debug, Clone, Default)]
pub(super) struct Located {
    pub found: Vec<(MessageId, u32)>,
    pub absent: Vec<MessageId>,
}

/// Regroupe les apparitions visées par nom de boîte, pour ne sélectionner chaque boîte qu'une fois
pub(super) fn wanted_by_mailbox(
    messages: &[ServerMessage],
    scope: &Scope,
) -> BTreeMap<String, Vec<Wanted>> {
    let mut groups: BTreeMap<String, Vec<Wanted>> = BTreeMap::new();
    for message in messages {
        for appearance in &message.appearances {
            let name = &appearance.mailbox_name;
            let wanted = match scope {
                Scope::All => true,
                Scope::Except(skipped) => name != skipped,
                Scope::Only(only) => name == only,
            };
            if wanted {
                groups.entry(name.clone()).or_default().push(Wanted {
                    message: message.id.clone(),
                    uid: appearance.uid,
                    validity: appearance.uid_validity,
                    header: message.header.clone(),
                });
            }
        }
    }
    groups
}

/// Sélectionne la boîte ; une boîte absente du serveur donne `None`
async fn select_if_present(
    client: &mut ImapClient,
    mailbox: &str,
) -> Result<Option<SelectedMailbox>, MiraError> {
    match client.select(mailbox, None).await {
        Ok(selected) => Ok(Some(selected)),
        Err(error) if error.code == server::MAILBOX_MISSING.0 => Ok(None),
        Err(error) => Err(error),
    }
}

/// Retrouve un message : par son UID s'il est encore valable, sinon par son `Message-ID` s'il est unique (un doublon est ambigu)
async fn find_uid(
    client: &mut ImapClient,
    item: &Wanted,
    present: &[u32],
    validity: u32,
) -> Result<Option<u32>, MiraError> {
    let trusted = item.validity.is_none_or(|known| known == validity);
    if item.uid != 0 && trusted && present.contains(&item.uid) {
        return Ok(Some(item.uid));
    }
    let Some(header) = &item.header else {
        return Ok(None);
    };
    let candidates = client.uids_with_message_id(header).await?;
    Ok(match candidates.as_slice() {
        [only] => Some(*only),
        _ => None,
    })
}

/// Sélectionne la boîte et dit, pour chaque message cherché, son UID actuel ou son absence
pub(super) async fn locate(
    client: &mut ImapClient,
    mailbox: &str,
    wanted: &[Wanted],
) -> Result<Located, MiraError> {
    let mut located = Located::default();
    let Some(selected) = select_if_present(client, mailbox).await? else {
        located.absent = wanted.iter().map(|item| item.message.clone()).collect();
        return Ok(located);
    };
    let known: Vec<u32> = wanted
        .iter()
        .map(|item| item.uid)
        .filter(|uid| *uid != 0)
        .collect();
    let present = client.existing_uids(&known).await?;
    for item in wanted {
        match find_uid(client, item, &present, selected.uid_validity).await? {
            Some(uid) => located.found.push((item.message.clone(), uid)),
            None => located.absent.push(item.message.clone()),
        }
    }
    Ok(located)
}
