//! Descente par étapes d'un compte (D-20) : boîtes, en-têtes, corps préchargés, ménage des orphelins.

mod account;
mod body;
mod inbox;
mod mailbox_list;
mod names;
mod order;
mod orphans;
mod prefetch;
mod progress;

pub use account::sync_account;
pub use inbox::sync_inbox;
pub use progress::{count, report_progress};
