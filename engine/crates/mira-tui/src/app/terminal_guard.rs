//! Garde du terminal : mode brut et écran alternatif, restaurés par `Drop` même en cas d'erreur.

use std::io::stdout;

use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};

use super::error::TuiError;
use super::terminal_probe::TerminalProbe;

/// Possède le terminal tant qu'il vit ; sa destruction rend le terminal à l'état normal.
#[derive(Debug)]
pub struct TerminalGuard {
    probe: Option<TerminalProbe>,
}

impl TerminalGuard {
    /// Active le mode brut et l'écran alternatif.
    pub fn enter() -> Result<Self, TuiError> {
        enable_raw_mode()?;
        if let Err(error) = execute!(stdout(), EnterAlternateScreen) {
            let _ = disable_raw_mode();
            return Err(error.into());
        }
        Ok(Self { probe: None })
    }

    /// Garde factice pour les tests : n'active rien, enregistre l'entrée et la restauration.
    pub fn for_tests() -> (Self, TerminalProbe) {
        let probe = TerminalProbe::default();
        probe.record_enter();
        (
            Self {
                probe: Some(probe.clone()),
            },
            probe,
        )
    }
}

impl Drop for TerminalGuard {
    /// Restaure le terminal ; les échecs sont ignorés car on ne peut plus les signaler.
    fn drop(&mut self) {
        match &self.probe {
            Some(probe) => probe.record_restore(),
            None => {
                let _ = execute!(stdout(), LeaveAlternateScreen);
                let _ = disable_raw_mode();
            }
        }
    }
}
