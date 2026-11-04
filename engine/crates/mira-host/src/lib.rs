//! Hôte du moteur : logique commune au pont FFI et au démon.
//! Un `Host` possède le moteur ouvert ; chaque processus en crée autant qu'il en a besoin.

pub mod envelope;
mod host;
mod lifecycle;

pub use host::Host;
