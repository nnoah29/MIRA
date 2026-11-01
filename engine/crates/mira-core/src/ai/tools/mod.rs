//! Outils en lecture seule offerts au modèle : catalogue, exécution par requêtes du contrat, rendu compact.
//! Chaque outil traduit des arguments JSON en requête de niveau `Read`, jamais en écriture.

mod args;
pub mod body;
mod catalog;
mod compose_draft;
mod edit_draft;
mod execute;
mod get_message;
mod get_thread;
mod instant;
mod list_identities;
mod list_mailboxes;
mod list_messages;
mod message_view;
mod messages;
mod proposal;
mod proposal_text;
mod propose;
pub mod render;
mod run;
mod run_result;
mod search_messages;

pub use catalog::{is_proposal, kind_of, specs, specs_for, summary_of};
pub use execute::is_read;
pub use propose::propose;
pub use run::run;
pub use run_result::ToolRun;
