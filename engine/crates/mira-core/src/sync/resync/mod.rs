//! Renumérotation d'une boîte (UIDVALIDITY changé) : revalidation des apparitions sans perdre d'identité.

mod purge;
mod renumber;

pub use renumber::resync_mailbox;
