//! Couture du mode brut : réel en production, remplaçable dans les tests.

/// Active et désactive le mode brut du terminal.
pub trait RawMode: Send {
    /// Active le mode brut.
    fn enable(&mut self) -> std::io::Result<()>;

    /// Désactive le mode brut.
    fn disable(&mut self) -> std::io::Result<()>;
}

/// Mode brut du vrai terminal, via crossterm.
#[derive(Debug, Default)]
pub struct SystemRawMode;

impl RawMode for SystemRawMode {
    /// Active le mode brut du terminal courant.
    fn enable(&mut self) -> std::io::Result<()> {
        crossterm::terminal::enable_raw_mode()
    }

    /// Désactive le mode brut du terminal courant.
    fn disable(&mut self) -> std::io::Result<()> {
        crossterm::terminal::disable_raw_mode()
    }
}
