//! Écrivain factice dont toute écriture échoue.

use std::io::{Error, ErrorKind, Write};

/// Simule un terminal qui refuse l'écran alternatif.
#[derive(Debug)]
pub struct FailingWriter;

impl Write for FailingWriter {
    /// Échoue toujours.
    fn write(&mut self, _bytes: &[u8]) -> std::io::Result<usize> {
        Err(Error::new(ErrorKind::BrokenPipe, "terminal fermé"))
    }

    /// Échoue toujours.
    fn flush(&mut self) -> std::io::Result<()> {
        Err(Error::new(ErrorKind::BrokenPipe, "terminal fermé"))
    }
}
