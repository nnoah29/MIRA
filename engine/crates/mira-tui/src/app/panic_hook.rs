//! Crochet de panique : rend le terminal avant d'afficher une panique du fil principal ; les autres fils n'y touchent pas.

use std::io::{Write, stdout};

use crossterm::cursor::Show;
use crossterm::execute;
use crossterm::terminal::LeaveAlternateScreen;

use super::keyboard_flags::{keyboard_flags_pushed, pop_keyboard_flags, set_keyboard_flags_pushed};
use super::raw_mode::{RawMode, SystemRawMode};

/// Installe le crochet qui restaure le vrai terminal ; à appeler depuis le fil qui détient le terminal.
pub fn install_panic_hook() {
    install_panic_hook_with(restore_terminal);
}

/// Installe un crochet qui, pour une panique du fil appelant, appelle `restore` avant le crochet précédent.
pub fn install_panic_hook_with(restore: impl Fn() + Send + Sync + 'static) {
    let owner = std::thread::current().id();
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if std::thread::current().id() == owner {
            restore();
        }
        previous(info);
    }));
}

/// Rend le vrai terminal (protocole clavier retiré, écran normal, curseur visible, mode brut coupé), échecs ignorés.
pub fn restore_terminal() {
    let keyboard = keyboard_flags_pushed();
    set_keyboard_flags_pushed(false);
    restore_terminal_keys_on(&mut stdout(), &mut SystemRawMode, keyboard);
}

/// Quitte l'écran alternatif et réaffiche le curseur sur `out`, puis coupe le mode brut.
pub fn restore_terminal_on(out: &mut impl Write, raw: &mut impl RawMode) {
    restore_terminal_keys_on(out, raw, false);
}

/// Retire d'abord le protocole clavier étendu s'il est posé, puis restaure comme `restore_terminal_on`.
pub fn restore_terminal_keys_on(out: &mut impl Write, raw: &mut impl RawMode, keyboard: bool) {
    if keyboard {
        pop_keyboard_flags(out);
    }
    let _ = execute!(out, LeaveAlternateScreen, Show);
    let _ = raw.disable();
}
