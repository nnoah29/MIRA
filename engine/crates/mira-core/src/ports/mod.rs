//! Dépendances injectables : horloge, identifiants, trousseau (annexe H).

pub mod clock;
pub mod ids;
pub mod resources;
pub mod resources_linux;
pub mod secrets;

pub use clock::{Clock, ManualClock, SystemClock};
pub use ids::{IdGenerator, SequentialIds, UuidV7Ids};
pub use resources::{FixedResources, SystemResources};
pub use resources_linux::LinuxResources;
pub use secrets::{
    ChunkedSecretStore, KeyringSecretStore, MemorySecretStore, SecretStore, SecretValue,
    run_blocking_call,
};
