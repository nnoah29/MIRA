#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Rapport d'activité d'un compte : passes, dernier succès, dernière erreur, boîtes en cours.

use mira_core::sync::{ReportHandle, SyncReport};
use mira_vocabulary::error::codes::network;
use mira_vocabulary::{ErrorCategory, MailboxId, MiraError, Timestamp};

/// Erreur réseau de test
fn cut() -> MiraError {
    MiraError::new(ErrorCategory::Network, network::CONNECTION_LOST, "coupé")
}

/// Une passe réussie note son instant et efface l'erreur ; une passe en échec garde le dernier succès et note le code
#[test]
fn passes_record_last_success_and_last_error() {
    let report = ReportHandle::default();
    assert_eq!(report.snapshot(), SyncReport::default());
    report.pass_started();
    assert!(report.snapshot().running);
    report.pass_ended(Timestamp::from_millis(10), &Ok(()));
    let done = report.snapshot();
    assert!(!done.running && done.last_error.is_none());
    assert_eq!(done.last_success, Some(Timestamp::from_millis(10)));
    report.pass_started();
    report.pass_ended(Timestamp::from_millis(20), &Err(cut()));
    let failed = report.snapshot();
    assert_eq!(failed.last_success, Some(Timestamp::from_millis(10)));
    assert_eq!(
        failed.last_error.as_deref(),
        Some("network.connection_lost")
    );
}

/// Les boîtes en cours apparaissent pendant leur synchronisation et disparaissent à la fin de la passe ou d'une rupture
#[test]
fn mailboxes_in_progress_follow_the_work() {
    let report = ReportHandle::default();
    let (inbox, archive) = (MailboxId::from_raw("inbox"), MailboxId::from_raw("archive"));
    report.pass_started();
    report.mailbox_started(&inbox);
    report.mailbox_started(&archive);
    assert_eq!(
        report.snapshot().mailboxes,
        [inbox.clone(), archive.clone()]
    );
    report.mailbox_ended(&inbox);
    assert_eq!(report.snapshot().mailboxes, [archive]);
    report.broke(&cut());
    let broken = report.snapshot();
    assert!(broken.mailboxes.is_empty() && !broken.running);
    assert_eq!(
        broken.last_error.as_deref(),
        Some("network.connection_lost")
    );
}

/// Deux poignées clonées partagent le même rapport
#[test]
fn cloned_handles_share_one_report() {
    let writer = ReportHandle::default();
    let reader = writer.clone();
    writer.pass_started();
    assert!(reader.snapshot().running);
}
