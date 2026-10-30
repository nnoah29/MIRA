//! Outils en lecture seule offerts au modèle : catalogue, exécution par requêtes du contrat, rendu compact.
//! Chaque outil traduit des arguments JSON en requête de niveau `Read`, jamais en écriture.

mod args;
pub mod body;
mod catalog;
mod execute;
mod get_message;
mod get_thread;
mod instant;
mod list_mailboxes;
mod list_messages;
mod message_view;
mod messages;
pub mod render;
mod run;
mod run_result;
mod search_messages;

pub use catalog::{kind_of, specs, summary_of};
pub use execute::is_read;
pub use run::run;
pub use run_result::ToolRun;
