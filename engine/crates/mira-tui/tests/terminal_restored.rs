//! Le terminal est restauré quand le garde est détruit : sortie d'écran avant fin du mode brut.

mod support;

use mira_tui::app::{TerminalGuard, TuiError};
use support::event_log::EventLog;
use support::failing_writer::FailingWriter;
use support::recording_raw::RecordingRaw;
use support::recording_writer::RecordingWriter;

const RESTORED: [&str; 4] = ["raw_on", "screen_enter", "screen_leave", "raw_off"];

/// Construit un garde dont les effets sont journalisés.
fn recorded_guard(log: &EventLog) -> TerminalGuard {
    let out = RecordingWriter { log: log.clone() };
    let raw = RecordingRaw { log: log.clone() };
    let Ok(guard) = TerminalGuard::with_parts(Box::new(out), Box::new(raw)) else {
        std::panic::resume_unwind(Box::new("entrée impossible"));
    };
    guard
}

/// Détruire le garde quitte l'écran alternatif avant de désactiver le mode brut.
#[test]
fn guard_restores_raw_mode_on_drop() {
    let log = EventLog::default();
    let guard = recorded_guard(&log);
    assert_eq!(log.snapshot(), ["raw_on", "screen_enter"]);
    drop(guard);
    assert_eq!(log.snapshot(), RESTORED);
}

/// Une panique pendant que le garde est détenu restaure quand même le terminal.
#[test]
fn guard_restores_when_holder_panics() {
    let log = EventLog::default();
    let guard = recorded_guard(&log);
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
        let _held = guard;
        std::panic::resume_unwind(Box::new("échec simulé"));
    }));
    assert!(outcome.is_err());
    assert_eq!(log.snapshot(), RESTORED);
}

/// Un retour anticipé en erreur depuis la fonction qui détient le garde restaure le terminal.
#[test]
fn guard_restores_when_holder_returns_error() {
    let log = EventLog::default();
    let guard = recorded_guard(&log);
    let result = failing_holder(guard);
    assert!(matches!(result, Err(TuiError::Terminal(_))));
    assert_eq!(log.snapshot(), RESTORED);
}

/// Détient le garde puis échoue.
fn failing_holder(_guard: TerminalGuard) -> Result<(), TuiError> {
    Err(TuiError::Terminal(std::io::Error::other("échec simulé")))
}

/// Si l'écran alternatif échoue, le mode brut déjà activé est désactivé et l'erreur est rendue.
#[test]
fn enter_failure_disables_raw_mode() {
    let log = EventLog::default();
    let raw = RecordingRaw { log: log.clone() };
    let result = TerminalGuard::with_parts(Box::new(FailingWriter), Box::new(raw));
    assert!(matches!(result, Err(TuiError::Terminal(_))));
    assert_eq!(log.snapshot(), ["raw_on", "raw_off"]);
}
