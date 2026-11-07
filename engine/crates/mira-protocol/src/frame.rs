//! Les trames échangées sur une connexion ; le champ `type` désigne la variante.

use mira_vocabulary::ContractVersion;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{Auth, HelloRefusal, HostOp, HostState, ProtocolVersion};

/// Raison de la fin d'un flux.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EndReason {
    Finished,
    Cancelled,
    Lagged,
    Error { code: String, message: String },
}

/// Une trame du protocole ; les échanges portent un `id` choisi par le client.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Frame {
    Hello {
        protocol_version: ProtocolVersion,
        contract_version: ContractVersion,
        client_id: String,
        client_name: String,
        auth: Auth,
    },
    HelloOk {
        protocol_version: ProtocolVersion,
        contract_version: ContractVersion,
        host_state: HostState,
    },
    HelloErr {
        reason: HelloRefusal,
    },
    Call {
        id: u64,
        request: Value,
    },
    Result {
        id: u64,
        envelope: Value,
    },
    OpenEvents {
        id: u64,
        after: Option<u64>,
        credit: u32,
    },
    OpenChat {
        id: u64,
        request: Value,
        credit: u32,
    },
    GetOriginalHtml {
        id: u64,
        message: String,
    },
    HostOp {
        id: u64,
        op: HostOp,
    },
    Event {
        id: u64,
        event: Value,
    },
    ChatDelta {
        id: u64,
        line: Value,
    },
    Credit {
        id: u64,
        n: u32,
    },
    Cancel {
        id: u64,
    },
    End {
        id: u64,
        reason: EndReason,
    },
    Ping {
        nonce: u64,
    },
    Pong {
        nonce: u64,
    },
    /// Trame d'un type que cette version ne connaît pas (pair plus récent) : à ignorer.
    #[serde(other)]
    Unknown,
}
