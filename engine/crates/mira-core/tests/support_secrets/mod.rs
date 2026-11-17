//! Trousseau factice plafonné et accessoires des tests de secrets longs.

mod fixtures;
mod limited;

pub use fixtures::{chunked, key, reference};
pub use limited::Raw;
