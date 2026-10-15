//! Protocole clavier étendu (Ctrl+Entrée réel) : posé à l'entrée s'il est pris en charge, retiré à toute restauration.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod support;

use mira_tui::app::TerminalGuard;
use mira_tui::app::panic_hook::restore_terminal_keys_on;
use support::event_log::EventLog;
use support::recording_raw::RecordingRaw;
use support::recording_writer::RecordingWriter;

const ENTERED: [&str; 3] = ["raw_on", "screen_enter", "keys_push"];
const RESTORED: [&str; 6] = [
    "raw_on",
    "screen_enter",
    "keys_push",
    "keys_pop",
    "screen_leave",
    "raw_off",
];

/// Garde journalisé ; `enhance` dit si le terminal prend en charge le protocole clavier étendu.
fn keyboard_guard(log: &EventLog, enhance: bool) -> TerminalGuard {
    let out = RecordingWriter { log: log.clone() };
    let raw = RecordingRaw { log: log.clone() };
    TerminalGuard::with_keyboard(Box::new(out), Box::new(raw), enhance).unwrap()
}

/// Pris en charge : posé après l'écran alternatif, retiré avant de le quitter.
#[test]
fn keyboard_flags_are_pushed_then_popped_on_restore() {
    let log = EventLog::default();
    let guard = keyboard_guard(&log, true);
    assert_eq!(log.snapshot(), ENTERED);
    drop(guard);
    assert_eq!(log.snapshot(), RESTORED);
}

/// Panique du fil qui détient le terminal : le protocole est quand même retiré.
#[test]
fn keyboard_flags_are_popped_when_the_holder_panics() {
    let log = EventLog::default();
    let guard = keyboard_guard(&log, true);
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
        let _held = guard;
        std::panic::resume_unwind(Box::new("échec simulé"));
    }));
    assert!(outcome.is_err());
    assert_eq!(log.snapshot(), RESTORED);
}

/// Non pris en charge : rien n'est posé ni retiré.
#[test]
fn unsupported_terminal_gets_no_keyboard_flags() {
    let log = EventLog::default();
    drop(keyboard_guard(&log, false));
    assert_eq!(
        log.snapshot(),
        ["raw_on", "screen_enter", "screen_leave", "raw_off"]
    );
}

/// Restauration du crochet de panique : retire le protocole s'il a été posé, avant l'écran et le mode brut.
#[test]
fn panic_restore_pops_keyboard_flags_first() {
    let log = EventLog::default();
    let mut out = RecordingWriter { log: log.clone() };
    let mut raw = RecordingRaw { log: log.clone() };
    restore_terminal_keys_on(&mut out, &mut raw, true);
    assert_eq!(log.snapshot(), ["keys_pop", "screen_leave", "raw_off"]);
    let quiet = EventLog::default();
    let mut out = RecordingWriter { log: quiet.clone() };
    let mut raw = RecordingRaw { log: quiet.clone() };
    restore_terminal_keys_on(&mut out, &mut raw, false);
    assert_eq!(quiet.snapshot(), ["screen_leave", "raw_off"]);
}
