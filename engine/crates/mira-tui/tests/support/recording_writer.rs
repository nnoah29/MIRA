//! Écrivain factice qui note l'entrée et la sortie de l'écran alternatif.

use std::io::Write;

use super::event_log::EventLog;

/// Reconnaît les séquences d'écran alternatif et les inscrit au journal.
#[derive(Debug)]
pub struct RecordingWriter {
    pub log: EventLog,
}

impl Write for RecordingWriter {
    /// Note `screen_enter` ou `screen_leave` selon la séquence reçue.
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let text = String::from_utf8_lossy(bytes);
        if text.contains("\x1b[?1049h") {
            self.log.push("screen_enter");
        }
        if text.contains("\x1b[?1049l") {
            self.log.push("screen_leave");
        }
        Ok(bytes.len())
    }

    /// Rien à vider.
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
