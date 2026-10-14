//! Politique de synchro de fond par commande : un CLI de courte durée ne synchronise que sur demande.

use mira_assembly::sync::BackgroundSync;

use super::{account, sync};
use crate::arguments::Commands;

/// Mode de synchro de fond du moteur pour cette commande ; `configured` est le mode des dépendances.
pub fn background_sync_for(command: &Commands, configured: BackgroundSync) -> BackgroundSync {
    match command {
        Commands::Sync(command) => sync::background_mode(command, configured),
        Commands::Account(command) => account::background_mode(command, configured),
        Commands::Engine(_)
        | Commands::Provider(_)
        | Commands::Identity(_)
        | Commands::Auth(_)
        | Commands::Operation(_)
        | Commands::Mailbox(_)
        | Commands::Message(_)
        | Commands::Thread(_)
        | Commands::Search(_)
        | Commands::Maintenance(_)
        | Commands::Action(_)
        | Commands::Draft(_)
        | Commands::Queue(_)
        | Commands::Events(_) => BackgroundSync::Off,
    }
}
