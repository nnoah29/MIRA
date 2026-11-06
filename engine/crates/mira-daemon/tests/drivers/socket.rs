//! Chemin « socket » : le moteur est derrière un démon, atteint par le protocole de trames.

use mira_assembly::api::EngineConfig;
use mira_protocol::{Frame, HostOp};
use serde_json::Value;

use super::link::SocketLink;
use crate::contract::driver::Driver;
use crate::support::TestDaemon;

/// Les scénarios parlent au moteur par une socket Unix et le protocole de trames.
pub struct SocketDriver {
    link: SocketLink,
}

impl Driver for SocketDriver {
    /// Démon dont le moteur est ouvert sur la configuration, ou jamais ouvert sans elle
    async fn start(config: Option<EngineConfig>) -> Self {
        let daemon = match config {
            Some(config) => TestDaemon::start(config).await,
            None => TestDaemon::start_closed().await,
        };
        Self {
            link: SocketLink::attach(daemon).await,
        }
    }

    /// `call` apparié par identifiant
    async fn execute(&self, request_json: &str) -> String {
        let request: Value = serde_json::from_str(request_json).unwrap_or(Value::Null);
        let id = self.link.id();
        self.link.result_of(id, Frame::Call { id, request }).await
    }

    /// `open_events` avec une grande fenêtre, jusqu'à ce que `emit` refuse
    async fn pump_events(&self, after: Option<u64>, mut emit: impl FnMut(String) -> bool) {
        let id = self.link.id();
        let mut client = self.link.client.lock().await;
        client.open_events(id, after, u32::MAX >> 2).await;
        while let Some(frame) = client.recv_stream(id).await {
            if let Frame::Event { event, .. } = frame
                && !emit(event.to_string())
            {
                client.send(Frame::Cancel { id }).await;
                return;
            }
        }
    }

    /// `open_chat` ; `cancel` si `emit` refuse, puis attente de `end`
    async fn pump_chat(&self, request_json: &str, mut emit: impl FnMut(String) -> bool) {
        let id = self.link.id();
        let mut client = self.link.client.lock().await;
        client.open_chat(id, request_json, u32::MAX >> 2).await;
        let mut cancelled = false;
        while let Some(frame) = client.recv_stream(id).await {
            if let Frame::ChatDelta { line, .. } = frame
                && !cancelled
                && !emit(line.to_string())
            {
                client.send(Frame::Cancel { id }).await;
                cancelled = true;
            }
        }
    }

    /// `get_original_html`
    async fn original_html(&self, message: &str) -> String {
        let id = self.link.id();
        let frame = Frame::GetOriginalHtml {
            id,
            message: message.into(),
        };
        self.link.result_of(id, frame).await
    }

    /// `host_op shutdown`
    async fn close(&self) -> String {
        let id = self.link.id();
        self.link
            .result_of(
                id,
                Frame::HostOp {
                    id,
                    op: HostOp::Shutdown,
                },
            )
            .await
    }
}
