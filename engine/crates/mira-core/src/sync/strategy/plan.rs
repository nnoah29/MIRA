//! Choix du plan de synchronisation d'une boîte : première fois, renumérotation ou changements.

use mira_vocabulary::{Mailbox, MiraError, SyncMarker};

use crate::capabilities::{ChangeDetection, choose_change_detection};
use crate::imap::{ResyncFrom, SelectedMailbox};
use crate::sync::session::SyncSession;

/// Ce qu'il faut faire d'une boîte, d'après son repère et ce que le serveur annonce à l'ouverture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plan {
    Initial,
    Renumbered,
    Incremental(ChangeDetection),
}

/// Détection retenue : QRESYNC et CONDSTORE exigent un compteur de modifications dans le repère.
fn detection_for(session: &SyncSession, marker: Option<SyncMarker>) -> ChangeDetection {
    let chosen = choose_change_detection(&session.conn.capabilities);
    let has_modseq = marker.is_some_and(|known| known.highest_modseq.is_some());
    match chosen {
        ChangeDetection::FullCompare => chosen,
        _ if has_modseq => chosen,
        _ => ChangeDetection::FullCompare,
    }
}

/// Demande de resynchronisation rapide, seulement si la détection retenue est QRESYNC.
fn resync_request(detection: ChangeDetection, marker: Option<SyncMarker>) -> Option<ResyncFrom> {
    let known = marker?;
    match (detection, known.highest_modseq) {
        (ChangeDetection::Qresync, Some(modseq)) => Some(ResyncFrom {
            uid_validity: known.uid_validity,
            highest_modseq: modseq,
        }),
        _ => None,
    }
}

/// Ouvre la boîte (avec QRESYNC si possible) et décide du plan d'après son numéro de validité.
pub async fn open_mailbox(
    session: &mut SyncSession,
    mailbox: &Mailbox,
) -> Result<(SelectedMailbox, Plan), MiraError> {
    let detection = detection_for(session, mailbox.marker);
    let resync = resync_request(detection, mailbox.marker);
    let selected = session.conn.client.select(&mailbox.name, resync).await?;
    let plan = match mailbox.marker {
        None => Plan::Initial,
        Some(known) if known.uid_validity != selected.uid_validity => Plan::Renumbered,
        Some(_) => Plan::Incremental(detection),
    };
    Ok((selected, plan))
}
