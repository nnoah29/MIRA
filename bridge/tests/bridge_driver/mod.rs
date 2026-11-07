#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Chemin « pont FFI » du jeu de contrat : les appels passent par les fonctions fines du pont, sur son hôte global.

use mira_assembly::api::EngineConfig;
use mira_bridge::{original_html, runtime};
use tokio::sync::{Mutex, MutexGuard};

use crate::contract::driver::Driver;

/// L'hôte du pont est unique dans le processus : un seul scénario à la fois.
static SESSION: Mutex<()> = Mutex::const_new(());

/// Le pont, tenu en exclusivité le temps d'un scénario.
pub struct BridgeDriver {
    _guard: MutexGuard<'static, ()>,
}

impl Driver for BridgeDriver {
    /// Prend l'exclusivité, ferme tout moteur resté ouvert, puis ouvre le moteur s'il y a une configuration
    async fn start(config: Option<EngineConfig>) -> Self {
        let guard = SESSION.lock().await;
        runtime::close().await;
        if let Some(config) = config {
            let opened = runtime::open(config);
            assert!(opened.contains("\"ok\":true"), "{opened}");
        }
        Self { _guard: guard }
    }

    /// `runtime::execute`
    async fn execute(&self, request_json: &str) -> String {
        runtime::execute(request_json).await
    }

    /// `runtime::pump_events`
    async fn pump_events(&self, after: Option<u64>, emit: impl FnMut(String) -> bool) {
        runtime::pump_events(after, emit).await
    }

    /// `runtime::pump_chat`
    async fn pump_chat(&self, request_json: &str, emit: impl FnMut(String) -> bool) {
        runtime::pump_chat(request_json, emit).await
    }

    /// `original_html::fetch`
    async fn original_html(&self, message: &str) -> String {
        original_html::fetch(message).await
    }

    /// `runtime::close`
    async fn close(&self) -> String {
        runtime::close().await
    }
}
