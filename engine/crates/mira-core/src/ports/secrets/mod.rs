//! Trousseau de secrets : valeur masquée, trait, trousseau système et trousseau mémoire.

mod keyring_store;
mod memory;
mod memory_inspect;
mod store;
mod value;

pub use keyring_store::KeyringSecretStore;
pub use memory::MemorySecretStore;
pub use store::SecretStore;
pub use value::SecretValue;
