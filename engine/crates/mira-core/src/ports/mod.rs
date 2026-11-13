//! Dépendances injectables : horloge, identifiants, trousseau (annexe H).

pub mod clock;
mod host_probe;
pub mod ids;
pub mod resources;
pub mod resources_host;
pub mod secrets;

pub use clock::{Clock, ManualClock, SystemClock};
pub use ids::{IdGenerator, SequentialIds, UuidV7Ids};
pub use resources::{FixedResources, SystemResources};
pub use resources_host::HostResources;
pub use secrets::{
    ChunkedSecretStore, KeyringSecretStore, MemorySecretStore, SecretStore, SecretValue,
    run_blocking_call,
};
