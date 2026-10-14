//! Mode brut factice qui note ses activations et désactivations.

use mira_tui::app::RawMode;

use super::event_log::EventLog;

/// Inscrit `raw_on` et `raw_off` au journal.
#[derive(Debug)]
pub struct RecordingRaw {
    pub log: EventLog,
}

impl RawMode for RecordingRaw {
    /// Note l'activation.
    fn enable(&mut self) -> std::io::Result<()> {
        self.log.push("raw_on");
        Ok(())
    }

    /// Note la désactivation.
    fn disable(&mut self) -> std::io::Result<()> {
        self.log.push("raw_off");
        Ok(())
    }
}
