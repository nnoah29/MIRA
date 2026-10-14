//! Enregistreur d'événements qui coupe le faux serveur au milieu du téléchargement des en-têtes.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use mira_core::events::{EventSink, RecordingSink};
use mira_testkit::imap::FakeImapServer;
use mira_vocabulary::{Event, EventKind, MiraError, SyncStep};

/// Coupe toutes les connexions après le premier lot d'en-têtes intégré, une seule fois.
#[derive(Debug)]
pub struct CuttingSink {
    pub inner: Arc<RecordingSink>,
    server: Arc<FakeImapServer>,
    armed: AtomicBool,
}

impl CuttingSink {
    /// Crée l'enregistreur piégé, armé
    pub fn new(inner: Arc<RecordingSink>, server: Arc<FakeImapServer>) -> Self {
        Self {
            inner,
            server,
            armed: AtomicBool::new(true),
        }
    }

    /// Désarme le piège pour la suite du test
    pub fn disarm(&self) {
        self.armed.store(false, Ordering::SeqCst);
    }
}

impl EventSink for CuttingSink {
    /// Enregistre l'événement puis coupe le serveur après le premier lot d'en-têtes
    fn emit(&self, kind: EventKind) -> Result<Event, MiraError> {
        let first_batch_done = matches!(
            &kind,
            EventKind::SyncProgress { step: SyncStep::Headers, done, total, .. }
                if *done > 0 && done < total
        );
        let event = self.inner.emit(kind)?;
        if first_batch_done && self.armed.swap(false, Ordering::SeqCst) {
            self.server.drop_all_connections();
        }
        Ok(event)
    }
}
