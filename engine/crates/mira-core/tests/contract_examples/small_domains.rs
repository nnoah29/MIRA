//! Exemples des domaines moteur, providers, opérations et événements.

use mira_core::api::{EngineRequest, EventRequest, OperationRequest, ProviderRequest};

pub const NAMES: &[&str] = &[
    "engine.get_contract_info",
    "engine.get_status",
    "provider.list_providers",
    "provider.detect_provider",
    "operation.get_operation",
    "operation.cancel_operation",
    "event.get_events",
];

pub const EXAMPLES: &[&str] = &[
    r#"{"domain":"engine","request":{"op":"get_contract_info"}}"#,
    r#"{"domain":"engine","request":{"op":"get_status"}}"#,
    r#"{"domain":"provider","request":{"op":"list_providers"}}"#,
    r#"{"domain":"provider","request":{"op":"detect_provider","address":"a@b.c"}}"#,
    r#"{"domain":"operation","request":{"op":"get_operation","operation":"o"}}"#,
    r#"{"domain":"operation","request":{"op":"cancel_operation","operation":"o"}}"#,
    r#"{"domain":"event","request":{"op":"get_events","filter":{"accounts":[],"categories":[],"after":null},"limit":10}}"#,
];

/// Nom `domaine.opération` d'une requête du domaine moteur
pub fn engine_name(request: &EngineRequest) -> &'static str {
    match request {
        EngineRequest::GetContractInfo => "engine.get_contract_info",
        EngineRequest::GetStatus => "engine.get_status",
    }
}

/// Nom `domaine.opération` d'une requête du domaine providers
pub fn provider_name(request: &ProviderRequest) -> &'static str {
    match request {
        ProviderRequest::ListProviders => "provider.list_providers",
        ProviderRequest::DetectProvider { .. } => "provider.detect_provider",
    }
}

/// Nom `domaine.opération` d'une requête du domaine opérations
pub fn operation_name(request: &OperationRequest) -> &'static str {
    match request {
        OperationRequest::GetOperation { .. } => "operation.get_operation",
        OperationRequest::CancelOperation { .. } => "operation.cancel_operation",
    }
}

/// Nom `domaine.opération` d'une requête du domaine événements
pub fn event_name(request: &EventRequest) -> &'static str {
    match request {
        EventRequest::GetEvents { .. } => "event.get_events",
    }
}
