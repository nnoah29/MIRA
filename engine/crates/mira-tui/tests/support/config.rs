//! Configuration de moteur de test : profil sous un dossier jetable, faux trousseau, horloge manuelle.

use std::path::Path;
use std::sync::Arc;

use mira_assembly::accounts::NetworkPolicy;
use mira_assembly::api::EngineConfig;
use mira_assembly::auth::OAuthClientConfig;
use mira_assembly::build_registry;
use mira_assembly::mira_vocabulary::{ProfileId, Timestamp};
use mira_assembly::ports::{ManualClock, MemorySecretStore, SequentialIds};
use mira_assembly::profile::ProfileEnvironment;

/// Configuration injectée au moteur : tout vit sous `home`, rien ne touche au système.
pub fn test_config(home: &Path, network: NetworkPolicy) -> EngineConfig {
    EngineConfig {
        profile: ProfileId::from_raw("default"),
        environment: ProfileEnvironment {
            mira_home: Some(home.to_path_buf()),
            xdg_data_home: None,
            home: None,
        },
        registry: build_registry(),
        secrets: Arc::new(MemorySecretStore::new()),
        clock: Arc::new(ManualClock::new(Timestamp::from_millis(1_700_000_000_000))),
        ids: Arc::new(SequentialIds::new()),
        oauth: OAuthClientConfig::default(),
        network,
    }
}
