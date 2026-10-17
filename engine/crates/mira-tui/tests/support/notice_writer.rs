//! Écrivain factice de l'avis de fermeture : note le texte et si le moteur vit encore à ce moment.

use std::io::Write;
use std::sync::Weak;

use mira_assembly::api::Engine;

use super::event_log::EventLog;

/// Inscrit `notice:<texte>:<moteur ouvert|moteur fermé>` au journal à chaque écriture.
#[derive(Debug)]
pub struct NoticeWriter {
    pub log: EventLog,
    pub engine: Weak<Engine>,
}

impl Write for NoticeWriter {
    /// Note le texte reçu et l'état du moteur.
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let alive = match self.engine.upgrade() {
            Some(_) => "moteur ouvert",
            None => "moteur fermé",
        };
        let text = String::from_utf8_lossy(bytes);
        self.log
            .push(&format!("notice:{}:{alive}", text.trim_end()));
        Ok(bytes.len())
    }

    /// Rien à vider.
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
