//! Crochet de panique : rend le terminal (avant l'affichage) seulement pour une panique du fil principal.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod support;

use mira_tui::app::panic_hook::{install_panic_hook_with, restore_terminal_on};
use support::event_log::EventLog;
use support::recording_raw::RecordingRaw;
use support::recording_writer::RecordingWriter;

/// Fil secondaire : pas de restauration ; fil qui a installé le crochet : restauration puis affichage.
#[test]
fn panic_hook_restores_only_on_main_thread_before_printing() {
    let log = EventLog::default();
    let printer = log.clone();
    std::panic::set_hook(Box::new(move |_| printer.push("print")));
    let restorer = log.clone();
    install_panic_hook_with(move || restorer.push("restore"));
    let worker = std::thread::spawn(|| panic!("échec simulé sur un fil secondaire"));
    assert!(worker.join().is_err());
    assert_eq!(log.snapshot(), ["print"]);
    let outcome = std::panic::catch_unwind(|| panic!("échec simulé"));
    let _ = std::panic::take_hook();
    assert!(outcome.is_err());
    assert_eq!(log.snapshot(), ["print", "restore", "print"]);
}

/// La restauration quitte l'écran alternatif puis coupe le mode brut.
#[test]
fn restore_terminal_leaves_screen_then_disables_raw_mode() {
    let log = EventLog::default();
    let mut out = RecordingWriter { log: log.clone() };
    let mut raw = RecordingRaw { log: log.clone() };
    restore_terminal_on(&mut out, &mut raw);
    assert_eq!(log.snapshot(), ["screen_leave", "raw_off"]);
}
