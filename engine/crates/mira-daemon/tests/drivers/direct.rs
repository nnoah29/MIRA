//! Chemin « hôte direct » : le moteur est ouvert dans le processus du test, sans transport.

use mira_assembly::api::EngineConfig;
use mira_host::Host;

use crate::contract::driver::Driver;

/// Un hôte du processus, comme le fait le pont FFI.
#[derive(Debug)]
pub struct DirectDriver {
    host: Host,
}

impl Driver for DirectDriver {
    /// Hôte neuf, ouvert sur la configuration si elle est donnée
    async fn start(config: Option<EngineConfig>) -> Self {
        let host = Host::new();
        if let Some(config) = config {
            let opened = host.open(config);
            assert!(opened.contains("\"ok\":true"), "{opened}");
        }
        Self { host }
    }

    /// Exécute sur l'hôte
    async fn execute(&self, request_json: &str) -> String {
        self.host.execute(request_json).await
    }

    /// Événements de l'hôte
    async fn pump_events(&self, after: Option<u64>, emit: impl FnMut(String) -> bool) {
        self.host.pump_events(after, emit).await
    }

    /// Génération de l'hôte
    async fn pump_chat(&self, request_json: &str, emit: impl FnMut(String) -> bool) {
        self.host.pump_chat(request_json, emit).await
    }

    /// HTML d'origine lu par l'hôte
    async fn original_html(&self, message: &str) -> String {
        self.host.original_html(message).await
    }

    /// Ferme le moteur de l'hôte
    async fn close(&self) -> String {
        self.host.close().await
    }
}
