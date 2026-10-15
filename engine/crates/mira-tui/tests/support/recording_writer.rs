//! Écrivain factice qui note l'écran alternatif et le protocole clavier étendu.

use std::io::Write;

use super::event_log::EventLog;

/// Reconnaît les séquences d'écran et de protocole clavier et les inscrit au journal.
#[derive(Debug)]
pub struct RecordingWriter {
    pub log: EventLog,
}

impl Write for RecordingWriter {
    /// Note l'entrée ou la sortie d'écran et la pose ou le retrait du protocole clavier étendu.
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let text = String::from_utf8_lossy(bytes);
        if text.contains("\x1b[?1049h") {
            self.log.push("screen_enter");
        }
        if text.contains("\x1b[?1049l") {
            self.log.push("screen_leave");
        }
        if text.contains("\x1b[>1u") {
            self.log.push("keys_push");
        }
        if text.contains("\x1b[<1u") {
            self.log.push("keys_pop");
        }
        Ok(bytes.len())
    }

    /// Rien à vider.
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
