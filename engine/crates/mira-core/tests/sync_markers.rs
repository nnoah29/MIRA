#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Repères de synchronisation (invariant 20) : jamais avancés avant les données, reprise sans perte ni doublon.

mod support_sync;

use mira_testkit::imap::CapabilitySet;
use mira_vocabulary::{ErrorCategory, EventKind};
use support_sync::mirror::{mailbox_named, mirror_rows, stored_messages};
use support_sync::scenarios::cut_then_resume;
use support_sync::within;

/// Invariant 20 : une coupure au milieu des en-têtes laisse le repère en arrière ; la reprise ne perd ni ne duplique rien
#[tokio::test]
async fn invariant_20_marker_advanced_after_data() {
    within(async {
        let run = cut_then_resume(CapabilitySet::full(), 250).await;
        assert_eq!(run.failure.category, ErrorCategory::Network);
        assert!(
            run.marker_after_cut.is_none(),
            "repère avancé avant les données"
        );
        assert!(
            (100..250).contains(&run.rows_after_cut),
            "{}",
            run.rows_after_cut
        );
        let rows = mirror_rows(&run.world.db);
        assert_eq!(rows.len(), 250);
        let mut uids: Vec<u32> = rows.iter().map(|r| r.uid).collect();
        uids.dedup();
        assert_eq!(uids.len(), 250);
        assert_eq!(stored_messages(&run.world.db), 250);
        assert!(mailbox_named(&run.world.db, "INBOX").marker.is_some());
    })
    .await;
}

/// La coupure est signalée par `SyncFailed` ; la reprise se termine par `SyncFinished`
#[tokio::test]
async fn interrupted_pass_reports_failure_then_resumes_to_completion() {
    within(async {
        let run = cut_then_resume(CapabilitySet::full(), 130).await;
        let kinds = run.world.recording.kinds();
        let failed = kinds
            .iter()
            .filter(|k| matches!(k, EventKind::SyncFailed { .. }))
            .count();
        assert_eq!(failed, 1);
        assert!(matches!(kinds.last(), Some(EventKind::SyncFinished { .. })));
        assert!(
            !kinds
                .iter()
                .any(|k| matches!(k, EventKind::NewMessageReceived { .. }))
        );
    })
    .await;
}

/// Pour chaque jeu de capacités, la reprise après coupure aboutit au même miroir complet
#[tokio::test]
async fn resume_after_cut_gives_the_same_mirror_for_every_capability_set() {
    within(async {
        let sets = [
            CapabilitySet::full(),
            CapabilitySet::without_qresync(),
            CapabilitySet::without_move(),
            CapabilitySet::minimal(),
        ];
        let mut finals = Vec::new();
        for caps in sets {
            let run = cut_then_resume(caps, 120).await;
            assert!(run.marker_after_cut.is_none());
            finals.push(mirror_rows(&run.world.db));
        }
        assert_eq!(finals[0].len(), 120);
        assert!(finals.iter().all(|rows| rows == &finals[0]));
    })
    .await;
}
