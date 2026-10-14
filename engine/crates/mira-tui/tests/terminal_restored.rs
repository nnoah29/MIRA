//! Le terminal est restauré quand le garde est détruit (mode brut et écran alternatif).

use mira_tui::app::TerminalGuard;

/// Détruire le garde restaure le terminal.
#[test]
fn guard_restores_raw_mode_on_drop() {
    let (guard, probe) = TerminalGuard::for_tests();
    assert!(probe.entered());
    assert!(!probe.restored());
    drop(guard);
    assert!(probe.restored());
}

/// La restauration a lieu une seule fois, même quand le garde est détruit dans une fermeture.
#[test]
fn guard_restores_once_even_when_dropped_after_panic_free_unwind() {
    let (guard, probe) = TerminalGuard::for_tests();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
        let _held = guard;
    }));
    assert!(result.is_ok());
    assert_eq!(probe.restore_count(), 1);
}
