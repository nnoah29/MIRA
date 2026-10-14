//! Détection des changements d'une boîte selon la stratégie : QRESYNC, CONDSTORE, comparaison complète.

use mira_vocabulary::{Mailbox, MiraError, SyncMarker};

use super::changes::Changes;
use crate::imap::{FetchedFlags, SelectedMailbox};
use crate::sync::session::SyncSession;

/// QRESYNC : le serveur a déjà rendu disparitions et changements à l'ouverture ; reste à chercher les nouveaux.
pub async fn qresync_changes(
    session: &mut SyncSession,
    selected: &SelectedMailbox,
    marker: SyncMarker,
) -> Result<Changes, MiraError> {
    let new = if selected.uid_next > marker.uid_next {
        session.conn.client.uids_from(marker.uid_next).await?
    } else {
        Vec::new()
    };
    Ok(Changes {
        vanished: selected.vanished.clone(),
        changed: selected.changed.clone(),
        new,
    })
}

/// UID connus localement pour la boîte, par ordre croissant.
fn local_uids(session: &SyncSession, mailbox: &Mailbox) -> Result<Vec<u32>, MiraError> {
    let known = session.ctx.db.read(|tx| tx.uids_in_mailbox(&mailbox.id))?;
    Ok(known.into_iter().map(|(uid, _)| uid).collect())
}

/// Compare la liste des UID du serveur à celle du miroir : disparus, et nouveaux depuis le repère.
async fn list_differences(
    session: &mut SyncSession,
    local: &[u32],
    marker: SyncMarker,
) -> Result<(Vec<u32>, Vec<u32>), MiraError> {
    let server = session.conn.client.all_uids().await?;
    let vanished = local.iter().filter(|uid| !server.contains(uid)).copied();
    let fresh = server.iter().filter(|uid| **uid >= marker.uid_next);
    Ok((vanished.collect(), fresh.copied().collect()))
}

/// Détection commune à CONDSTORE (drapeaux modifiés seulement) et à la comparaison complète.
pub async fn compared_changes(
    session: &mut SyncSession,
    mailbox: &Mailbox,
    marker: SyncMarker,
    only_since_modseq: bool,
) -> Result<Changes, MiraError> {
    let local = local_uids(session, mailbox)?;
    let since = marker.highest_modseq.filter(|_| only_since_modseq);
    let changed: Vec<FetchedFlags> = session.conn.client.fetch_flags(&local, since).await?;
    let (vanished, new) = list_differences(session, &local, marker).await?;
    Ok(Changes {
        vanished,
        changed,
        new,
    })
}
