//! Description statique d'un provider.

use serde::{Deserialize, Serialize};

use super::limits::ProviderLimits;
use super::quirks::ProviderQuirks;
use crate::domain::auth::AuthMethod;
use crate::ids::keys::ProviderId;

/// Fiche d'identité d'un provider : domaines gérés, authentifications, limites, particularités.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderDescriptor {
    pub id: ProviderId,
    pub display_name: String,
    pub domains: Vec<String>,
    pub auth_methods: Vec<AuthMethod>,
    pub limits: ProviderLimits,
    pub quirks: ProviderQuirks,
}
