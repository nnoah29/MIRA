//! Le crochet de panique rend le terminal avant que la panique ne soit affichée.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod support;

use mira_tui::app::panic_hook::install_panic_hook_with;
use support::event_log::EventLog;

/// La restauration passe avant le crochet précédent, celui qui affiche le message.
#[test]
fn panic_hook_restores_terminal_before_printing() {
    let log = EventLog::default();
    let printer = log.clone();
    std::panic::set_hook(Box::new(move |_| printer.push("print")));
    let restorer = log.clone();
    install_panic_hook_with(move || restorer.push("restore"));
    let outcome = std::panic::catch_unwind(|| panic!("échec simulé"));
    let _ = std::panic::take_hook();
    assert!(outcome.is_err());
    assert_eq!(log.snapshot(), ["restore", "print"]);
}
