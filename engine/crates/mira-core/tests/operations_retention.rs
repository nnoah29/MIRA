#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Table des opérations : les opérations terminées les plus anciennes sont oubliées, les opérations en cours jamais.

use std::sync::Arc;

use mira_core::events::RecordingSink;
use mira_core::operations::{OperationRegistry, OperationStatus};
use mira_core::ports::SequentialIds;

/// Registre neuf et son enregistreur d'événements
fn registry() -> OperationRegistry {
    let sink = Arc::new(RecordingSink::default());
    OperationRegistry::new(Arc::new(SequentialIds::new()), sink)
}

/// Les opérations terminées au-delà de la borne sont oubliées, les plus récentes restent consultables
#[test]
fn regression_finished_operations_are_forgotten_beyond_the_bound() {
    let registry = registry();
    let mut first = None;
    let mut last = None;
    for _ in 0..600 {
        let handle = registry.start("fetch_message_body");
        handle.complete(Ok(()));
        first.get_or_insert_with(|| handle.id().clone());
        last = Some(handle.id().clone());
    }
    assert!(registry.get(&first.unwrap()).is_err());
    let newest = registry.get(&last.unwrap()).unwrap();
    assert_eq!(newest.status, OperationStatus::Finished);
}

/// Une opération encore en cours n'est jamais oubliée, même parmi des centaines d'opérations terminées
#[test]
fn running_operation_survives_the_cleanup() {
    let registry = registry();
    let running = registry.start("extend_window");
    for _ in 0..600 {
        registry.start("search_server").complete(Ok(()));
    }
    let kept = registry.get(running.id()).unwrap();
    assert!(matches!(kept.status, OperationStatus::Running { .. }));
    registry.cancel(running.id()).unwrap();
}
