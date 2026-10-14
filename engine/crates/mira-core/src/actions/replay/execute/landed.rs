//! Copies déjà arrivées dans la destination d'un déplacement (inv. 18) : même Message-ID, même taille, dates internes proches.

use std::collections::{BTreeMap, BTreeSet};

use mira_vocabulary::{MessageId, MiraError, Timestamp};

use crate::actions::replay::job::ReplayJob;
use crate::imap::ImapClient;

const DATE_TOLERANCE_MILLIS: i64 = 60_000;

/// Taille et date interne d'une copie d'un message sur le serveur.
#[derive(Debug, Clone, Copy)]
struct Trace {
    size: u64,
    date: Option<Timestamp>,
}

/// Traces des UID donnés dans la boîte sélectionnée
async fn traces_of(
    client: &mut ImapClient,
    uids: &[u32],
) -> Result<BTreeMap<u32, Trace>, MiraError> {
    if uids.is_empty() {
        return Ok(BTreeMap::new());
    }
    let fetched = client.fetch_headers(uids, &[]).await?;
    Ok(fetched
        .into_iter()
        .map(|found| {
            let trace = Trace {
                size: found.size,
                date: found.internal_date,
            };
            (found.uid, trace)
        })
        .collect())
}

/// Deux copies sont les mêmes si leur taille est égale et leurs dates internes proches (une date absente ne contredit pas)
fn same_copy(left: &Trace, right: &Trace) -> bool {
    let close = match (left.date, right.date) {
        (Some(a), Some(b)) => (a.as_millis() - b.as_millis()).abs() <= DATE_TOLERANCE_MILLIS,
        _ => true,
    };
    left.size == right.size && close
}

/// `Message-ID` d'en-tête d'un message visé par le travail
fn header_of<'a>(job: &'a ReplayJob, message: &MessageId) -> Option<&'a str> {
    let found = job.messages.iter().find(|item| &item.id == message)?;
    found.header.as_deref()
}

/// UID de la boîte source sélectionnée dont une copie identique existe déjà dans la destination ; la destination est sélectionnée ensuite
pub(super) async fn landed_sources(
    client: &mut ImapClient,
    job: &ReplayJob,
    found: &[(MessageId, u32)],
    destination: &str,
) -> Result<BTreeSet<u32>, MiraError> {
    let uids: Vec<u32> = found.iter().map(|(_, uid)| *uid).collect();
    let sources = traces_of(client, &uids).await?;
    client.select(destination, None).await?;
    let mut landed = BTreeSet::new();
    for (message, uid) in found {
        let (Some(header), Some(source)) = (header_of(job, message), sources.get(uid)) else {
            continue;
        };
        let candidates = client.uids_with_message_id(header).await?;
        let copies = traces_of(client, &candidates).await?;
        if copies.values().any(|copy| same_copy(copy, source)) {
            landed.insert(*uid);
        }
    }
    Ok(landed)
}

/// Messages disparus de la source qui ont déjà un exemplaire (même Message-ID) dans la destination
pub(super) async fn arrived_in_destination(
    client: &mut ImapClient,
    job: &ReplayJob,
    vanished: &[MessageId],
    destination: &str,
) -> Result<BTreeSet<MessageId>, MiraError> {
    let mut arrived = BTreeSet::new();
    if vanished.is_empty() {
        return Ok(arrived);
    }
    client.select(destination, None).await?;
    for message in vanished {
        let Some(header) = header_of(job, message) else {
            continue;
        };
        if !client.uids_with_message_id(header).await?.is_empty() {
            arrived.insert(message.clone());
        }
    }
    Ok(arrived)
}
