//! Renumérotation d'une boîte (UIDVALIDITY changé) : revalidation des apparitions sans perdre d'identité.

mod forced;
mod purge;
mod renumber;

pub use forced::force_resync;
pub use renumber::resync_mailbox;
