#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Opérations longues : identifiant, progression, fin, échec, annulation (12.2.4).

use std::sync::Arc;

use mira_core::events::RecordingSink;
use mira_core::operations::{OperationRegistry, OperationStatus};
use mira_core::ports::SequentialIds;
use mira_vocabulary::error::codes::{network, not_found};
use mira_vocabulary::{ErrorCategory, EventKind, MiraError, OperationId};

/// Construit un registre et l'enregistreur de ses événements
fn registry() -> (OperationRegistry, Arc<RecordingSink>) {
    let sink = Arc::new(RecordingSink::default());
    (
        OperationRegistry::new(Arc::new(SequentialIds::new()), sink.clone()),
        sink,
    )
}

/// La progression est consultable et émise, puis la fin est signalée
#[test]
fn progress_then_finish_is_visible_and_emitted() {
    let (registry, sink) = registry();
    let handle = registry.start("sync");
    handle.progress(3, 10);
    assert_eq!(
        registry.get(handle.id()).unwrap().status,
        OperationStatus::Running { done: 3, total: 10 }
    );
    handle.complete(Ok(()));
    assert_eq!(
        registry.get(handle.id()).unwrap().status,
        OperationStatus::Finished
    );
    assert!(matches!(
        sink.kinds().last(),
        Some(EventKind::OperationFinished { .. })
    ));
}

/// L'annulation est observée par la tâche et signalée comme telle
#[test]
fn cancellation_is_observed_and_reported() {
    let (registry, sink) = registry();
    let handle = registry.start("extend_window");
    assert!(!handle.is_cancelled());
    registry.cancel(handle.id()).unwrap();
    assert!(handle.is_cancelled());
    handle.complete(Ok(()));
    assert_eq!(
        registry.get(handle.id()).unwrap().status,
        OperationStatus::Cancelled
    );
    assert!(matches!(
        sink.kinds().last(),
        Some(EventKind::OperationCancelled { .. })
    ));
}

/// Un échec porte le code stable de l'erreur
#[test]
fn failure_carries_the_stable_code() {
    let (registry, _sink) = registry();
    let handle = registry.start("fetch_body");
    let error = MiraError::new(ErrorCategory::Network, network::OFFLINE, "hors ligne");
    handle.complete(Err(&error));
    let status = registry.get(handle.id()).unwrap().status;
    assert_eq!(
        status,
        OperationStatus::Failed {
            code: "network.offline".to_string()
        }
    );
}

/// Une opération inconnue est signalée par un code stable
#[test]
fn unknown_operation_is_not_found() {
    let (registry, _sink) = registry();
    let unknown = OperationId::from_raw("nope");
    assert_eq!(
        registry.get(&unknown).unwrap_err().code,
        not_found::OPERATION.0
    );
    assert_eq!(
        registry.cancel(&unknown).unwrap_err().code,
        not_found::OPERATION.0
    );
}
