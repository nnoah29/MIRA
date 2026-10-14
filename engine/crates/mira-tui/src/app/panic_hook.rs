//! Crochet de panique : rend le terminal avant d'afficher la panique, sinon le message serait illisible.

use std::io::stdout;

use crossterm::cursor::Show;
use crossterm::execute;
use crossterm::terminal::{LeaveAlternateScreen, disable_raw_mode};

/// Installe le crochet qui restaure le vrai terminal avant le crochet précédent.
pub fn install_panic_hook() {
    install_panic_hook_with(restore_terminal);
}

/// Installe un crochet qui appelle `restore` puis le crochet précédent (qui affiche la panique).
pub fn install_panic_hook_with(restore: impl Fn() + Send + Sync + 'static) {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore();
        previous(info);
    }));
}

/// Quitte l'écran alternatif, réaffiche le curseur et coupe le mode brut, en ignorant les échecs.
pub fn restore_terminal() {
    let _ = execute!(stdout(), LeaveAlternateScreen, Show);
    let _ = disable_raw_mode();
}
