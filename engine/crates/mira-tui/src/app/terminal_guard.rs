//! Garde du terminal : mode brut et écran alternatif, restaurés par `Drop` même en cas d'erreur.

use std::io::{Write, stdout};

use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};

use super::error::TuiError;
use super::raw_mode::{RawMode, SystemRawMode};

/// Possède le terminal tant qu'il vit ; sa destruction rend le terminal à l'état normal.
pub struct TerminalGuard {
    out: Box<dyn Write + Send>,
    raw: Box<dyn RawMode>,
}

impl std::fmt::Debug for TerminalGuard {
    /// Affiche le type sans exposer les flux.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("TerminalGuard")
    }
}

impl TerminalGuard {
    /// Active le mode brut et l'écran alternatif du vrai terminal.
    pub fn enter() -> Result<Self, TuiError> {
        Self::with_parts(Box::new(stdout()), Box::new(SystemRawMode))
    }

    /// Active le mode brut puis l'écran alternatif sur les flux donnés ; défait le mode brut si l'écran échoue.
    pub fn with_parts(
        mut out: Box<dyn Write + Send>,
        mut raw: Box<dyn RawMode>,
    ) -> Result<Self, TuiError> {
        raw.enable()?;
        if let Err(error) = execute!(out, EnterAlternateScreen) {
            let _ = raw.disable();
            return Err(error.into());
        }
        Ok(Self { out, raw })
    }
}

impl Drop for TerminalGuard {
    /// Quitte l'écran alternatif puis désactive le mode brut ; les échecs sont ignorés faute de recours.
    fn drop(&mut self) {
        let _ = execute!(self.out, LeaveAlternateScreen);
        let _ = self.raw.disable();
    }
}
