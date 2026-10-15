//! Garde du terminal : mode brut, écran alternatif et protocole clavier étendu, restaurés par `Drop` même en cas d'erreur.

use std::io::{Write, stdout};

use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};

use super::error::TuiError;
use super::keyboard_flags::{
    keyboard_flags_pushed, pop_keyboard_flags, push_keyboard_flags, set_keyboard_flags_pushed,
    terminal_supports_keyboard_flags,
};
use super::raw_mode::{RawMode, SystemRawMode};

/// Possède le terminal tant qu'il vit ; sa destruction rend le terminal à l'état normal.
pub struct TerminalGuard {
    out: Box<dyn Write + Send>,
    raw: Box<dyn RawMode>,
    keyboard: bool,
    real: bool,
}

impl std::fmt::Debug for TerminalGuard {
    /// Affiche le type sans exposer les flux.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("TerminalGuard")
    }
}

impl TerminalGuard {
    /// Active le mode brut, l'écran alternatif et, s'il est pris en charge, le protocole clavier étendu du vrai terminal.
    pub fn enter() -> Result<Self, TuiError> {
        let enhance = terminal_supports_keyboard_flags();
        let mut guard = Self::with_keyboard(Box::new(stdout()), Box::new(SystemRawMode), enhance)?;
        guard.real = true;
        set_keyboard_flags_pushed(guard.keyboard);
        Ok(guard)
    }

    /// Active le mode brut puis l'écran alternatif sur les flux donnés, sans protocole clavier étendu.
    pub fn with_parts(out: Box<dyn Write + Send>, raw: Box<dyn RawMode>) -> Result<Self, TuiError> {
        Self::with_keyboard(out, raw, false)
    }

    /// Mode brut, écran alternatif, puis protocole clavier étendu si `enhance` ; défait le mode brut si l'écran échoue.
    pub fn with_keyboard(
        mut out: Box<dyn Write + Send>,
        mut raw: Box<dyn RawMode>,
        enhance: bool,
    ) -> Result<Self, TuiError> {
        raw.enable()?;
        if let Err(error) = execute!(out, EnterAlternateScreen) {
            let _ = raw.disable();
            return Err(error.into());
        }
        let keyboard = enhance && push_keyboard_flags(&mut out).is_ok();
        Ok(Self {
            out,
            raw,
            keyboard,
            real: false,
        })
    }
}

impl Drop for TerminalGuard {
    /// Retire le protocole clavier encore posé (le crochet de panique a pu le faire), quitte l'écran alternatif puis désactive le mode brut ; échecs ignorés.
    fn drop(&mut self) {
        let pushed = if self.real {
            keyboard_flags_pushed()
        } else {
            self.keyboard
        };
        if pushed {
            pop_keyboard_flags(&mut self.out);
            set_keyboard_flags_pushed(false);
        }
        let _ = execute!(self.out, LeaveAlternateScreen);
        let _ = self.raw.disable();
    }
}
