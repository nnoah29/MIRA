#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! La configuration de production assemble les quatre providers sans règle réseau ni OAuth imposée.

use mira_assembly::mira_vocabulary::ProfileId;
use mira_assembly::production_config;

/// Le profil demandé, les quatre providers et des politiques vides
#[test]
fn production_config_has_real_providers_and_empty_policies() {
    let config = production_config(ProfileId::from_raw("work"));
    assert_eq!(config.profile.as_str(), "work");
    assert_eq!(config.registry.list().len(), 4);
    assert!(config.network.server_overrides.is_empty());
    assert!(config.oauth.client_ids.is_empty());
    assert!(config.oauth.endpoint_overrides.is_empty());
}
