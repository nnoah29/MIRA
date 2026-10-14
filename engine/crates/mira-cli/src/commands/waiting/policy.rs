//! Politique d'attente de chaque commande : un `match` exhaustif sur tous les domaines.

use super::WaitPolicy;
use crate::arguments::Commands;
use crate::commands::{action, draft, mailbox, message, queue, search};

/// Ce que le CLI attend avant de rendre la main pour cette commande.
pub fn policy_of(command: &Commands) -> WaitPolicy {
    match command {
        Commands::Mailbox(command) => mailbox::wait_policy(command),
        Commands::Message(command) => message::wait_policy(command),
        Commands::Search(command) => search::wait_policy(command),
        Commands::Action(command) => action::wait_policy(command),
        Commands::Draft(command) => draft::wait_policy(command),
        Commands::Queue(command) => queue::wait_policy(command),
        Commands::Engine(_)
        | Commands::Provider(_)
        | Commands::Account(_)
        | Commands::Identity(_)
        | Commands::Auth(_)
        | Commands::Operation(_)
        | Commands::Events(_)
        | Commands::Thread(_)
        | Commands::Maintenance(_) => WaitPolicy::Immediate,
    }
}
