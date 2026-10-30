//! Niveau de risque des requêtes : système, fournisseurs, opérations, événements, authentification et identités.

use mira_vocabulary::ToolRisk;

use crate::api::contract::{
    AuthRequest, EngineRequest, EventRequest, IdentityRequest, OperationRequest, ProviderRequest,
};

/// Niveau de risque des requêtes du domaine engine.
pub(super) fn engine_risk(request: &EngineRequest) -> ToolRisk {
    match request {
        EngineRequest::GetContractInfo | EngineRequest::GetStatus => ToolRisk::Read,
    }
}

/// Niveau de risque des requêtes du domaine provider.
pub(super) fn provider_risk(request: &ProviderRequest) -> ToolRisk {
    match request {
        ProviderRequest::ListProviders => ToolRisk::Read,
        ProviderRequest::DetectProvider { .. } => ToolRisk::Sensitive,
    }
}

/// Niveau de risque des requêtes du domaine operation.
pub(super) fn operation_risk(request: &OperationRequest) -> ToolRisk {
    match request {
        OperationRequest::GetOperation { .. } => ToolRisk::Read,
        OperationRequest::CancelOperation { .. } => ToolRisk::Reversible,
    }
}

/// Niveau de risque des requêtes du domaine event.
pub(super) fn event_risk(request: &EventRequest) -> ToolRisk {
    match request {
        EventRequest::GetEvents { .. } => ToolRisk::Read,
    }
}

/// Niveau de risque des requêtes du domaine auth.
pub(super) fn auth_risk(request: &AuthRequest) -> ToolRisk {
    match request {
        AuthRequest::StartOAuth { .. }
        | AuthRequest::CompleteOAuth { .. }
        | AuthRequest::CancelOAuth { .. }
        | AuthRequest::StoreAppPassword { .. }
        | AuthRequest::Reauthenticate { .. } => ToolRisk::Sensitive,
    }
}

/// Niveau de risque des requêtes du domaine identity.
pub(super) fn identity_risk(request: &IdentityRequest) -> ToolRisk {
    match request {
        IdentityRequest::ListIdentities { .. } => ToolRisk::Read,
        IdentityRequest::AddIdentity(..)
        | IdentityRequest::UpdateIdentity { .. }
        | IdentityRequest::SetDefaultIdentity { .. } => ToolRisk::Sensitive,
    }
}
