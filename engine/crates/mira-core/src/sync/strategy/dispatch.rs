//! Synchronisation d'une boîte : ouverture, plan, application, repère.

use mira_vocabulary::{LogLevel, Mailbox, MiraError};

use super::changes::apply_changes;
use super::finish::finish_mailbox;
use super::first::first_sync;
use super::plan::{Plan, open_mailbox};
use crate::capabilities::ChangeDetection;
use crate::imap::SelectedMailbox;
use crate::sync::resync::resync_mailbox;
use crate::sync::session::SyncSession;

/// Journalise le repli choisi quand QRESYNC n'est pas utilisable (10.14).
fn announce_fallback(session: &SyncSession, mailbox: &Mailbox, detection: ChangeDetection) {
    let text = match detection {
        ChangeDetection::Qresync => return,
        ChangeDetection::Condstore => {
            "QRESYNC indisponible : drapeaux modifiés (CONDSTORE) et liste des UID"
        }
        ChangeDetection::FullCompare => {
            "CONDSTORE indisponible : comparaison complète des drapeaux et des UID"
        }
    };
    let text = format!("boîte {} : {text}", mailbox.name);
    session.ctx.diagnose(LogLevel::Info, "sync.strategy", &text);
}

/// Applique le plan décidé à l'ouverture de la boîte.
async fn run_plan(
    session: &mut SyncSession,
    mailbox: &Mailbox,
    selected: &SelectedMailbox,
    plan: Plan,
) -> Result<(), MiraError> {
    match plan {
        Plan::Initial => first_sync(session, mailbox, selected).await,
        Plan::Renumbered => resync_mailbox(session, mailbox, selected).await,
        Plan::Incremental(detection) => {
            announce_fallback(session, mailbox, detection);
            apply_changes(session, mailbox, selected, detection).await
        }
    }
}

/// Synchronise une boîte avec le serveur : données d'abord, repère ensuite.
pub async fn sync_mailbox(session: &mut SyncSession, mailbox: &Mailbox) -> Result<(), MiraError> {
    let (selected, plan) = open_mailbox(session, mailbox).await?;
    run_plan(session, mailbox, &selected, plan).await?;
    finish_mailbox(session, mailbox, &selected)
}
