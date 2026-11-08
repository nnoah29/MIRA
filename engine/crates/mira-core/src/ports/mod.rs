//! Dépendances injectables : horloge, identifiants, trousseau (annexe H).

pub mod clock;
pub mod ids;
pub mod secrets;

pub use clock::{Clock, ManualClock, SystemClock};
pub use ids::{IdGenerator, SequentialIds, UuidV7Ids};
pub use secrets::{
    ChunkedSecretStore, KeyringSecretStore, MemorySecretStore, SecretStore, SecretValue,
    run_blocking_call,
};
