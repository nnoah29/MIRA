//! Profil : emplacements des données et verrou exclusif (D-10, D-27).

mod environment;
mod errors;
mod lock;
mod paths;

pub use environment::ProfileEnvironment;
pub use lock::ProfileLock;
pub use paths::ProfilePaths;
