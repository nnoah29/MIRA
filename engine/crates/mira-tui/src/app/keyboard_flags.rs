//! Protocole clavier étendu (Ctrl+Entrée distinct d'Entrée) : posé si le terminal le prend en charge, toujours retiré.

use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};

use crossterm::event::{
    KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::{Command, execute};

static PUSHED: AtomicBool = AtomicBool::new(false);

/// Vrai si le vrai terminal annonce la prise en charge du protocole ; faux s'il ne répond pas.
pub fn terminal_supports_keyboard_flags() -> bool {
    crossterm::terminal::supports_keyboard_enhancement().unwrap_or(false)
}

/// Pose le seul niveau « codes d'échappement sans ambiguïté » sur `out`, la séquence écrite d'un seul tenant.
pub fn push_keyboard_flags(out: &mut impl Write) -> std::io::Result<()> {
    let flags = KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES;
    let mut sequence = String::new();
    PushKeyboardEnhancementFlags(flags)
        .write_ansi(&mut sequence)
        .map_err(std::io::Error::other)?;
    out.write_all(sequence.as_bytes())?;
    out.flush()
}

/// Retire le niveau posé, en ignorant l'échec faute de recours.
pub fn pop_keyboard_flags(out: &mut impl Write) {
    let _ = execute!(out, PopKeyboardEnhancementFlags);
}

/// Mémorise, pour le crochet de panique, si le protocole est posé sur le vrai terminal.
pub fn set_keyboard_flags_pushed(pushed: bool) {
    PUSHED.store(pushed, Ordering::SeqCst);
}

/// Vrai si le protocole est posé sur le vrai terminal.
pub fn keyboard_flags_pushed() -> bool {
    PUSHED.load(Ordering::SeqCst)
}
