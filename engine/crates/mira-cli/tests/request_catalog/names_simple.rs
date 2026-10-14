//! Noms des variantes des petits domaines : le `match` sans `_` casse la compilation si une variante naît.

use mira_assembly::api::{EngineRequest, EventRequest, OperationRequest, ProviderRequest};

/// Nom d'une requête du domaine moteur.
pub fn engine_name(request: &EngineRequest) -> &'static str {
    match request {
        EngineRequest::GetContractInfo => "engine.get_contract_info",
        EngineRequest::GetStatus => "engine.get_status",
    }
}

/// Nom d'une requête du domaine providers.
pub fn provider_name(request: &ProviderRequest) -> &'static str {
    match request {
        ProviderRequest::ListProviders => "provider.list_providers",
        ProviderRequest::DetectProvider { .. } => "provider.detect_provider",
    }
}

/// Nom d'une requête du domaine opérations.
pub fn operation_name(request: &OperationRequest) -> &'static str {
    match request {
        OperationRequest::GetOperation { .. } => "operation.get_operation",
        OperationRequest::CancelOperation { .. } => "operation.cancel_operation",
    }
}

/// Nom d'une requête du domaine événements.
pub fn event_name(request: &EventRequest) -> &'static str {
    match request {
        EventRequest::GetEvents { .. } => "event.get_events",
    }
}
