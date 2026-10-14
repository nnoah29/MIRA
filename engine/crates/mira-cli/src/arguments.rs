//! Définition de la ligne de commande `mira` (clap) : options globales et un domaine par variante.

use clap::{ArgAction, Parser, Subcommand};

use crate::commands::account::AccountCommand;
use crate::commands::action::ActionCommand;
use crate::commands::auth::AuthCommand;
use crate::commands::draft::DraftCommand;
use crate::commands::engine::EngineCommand;
use crate::commands::events::EventsArgs;
use crate::commands::identity::IdentityCommand;
use crate::commands::mailbox::MailboxCommand;
use crate::commands::maintenance::MaintenanceCommand;
use crate::commands::message::MessageCommand;
use crate::commands::operation::OperationCommand;
use crate::commands::provider::ProviderCommand;
use crate::commands::queue::QueueCommand;
use crate::commands::search::SearchCommand;
use crate::commands::thread::ThreadCommand;

/// Client en ligne de commande de MIRA.
#[derive(Debug, Parser)]
#[command(
    name = "mira",
    version,
    about = "Client mail MIRA en ligne de commande"
)]
pub struct Cli {
    /// Écrit le résultat sous forme d'enveloppe JSON versionnée.
    #[arg(long, global = true)]
    pub json: bool,
    /// Nom du profil (dossier de données) à utiliser.
    #[arg(long, global = true, default_value = "default")]
    pub profile: String,
    /// Journalisation plus bavarde sur stderr (`-v` info, `-vv` debug).
    #[arg(short = 'v', long = "verbose", global = true, action = ArgAction::Count)]
    pub verbose: u8,
    #[command(subcommand)]
    pub command: Commands,
}

/// Domaines de commandes, un par domaine du contrat d'API.
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Informations sur le moteur.
    #[command(subcommand)]
    Engine(EngineCommand),
    /// Catalogue des providers et détection.
    #[command(subcommand)]
    Provider(ProviderCommand),
    /// Comptes de messagerie.
    #[command(subcommand)]
    Account(AccountCommand),
    /// Identités d'envoi.
    #[command(subcommand)]
    Identity(IdentityCommand),
    /// Authentification et secrets.
    #[command(subcommand)]
    Auth(AuthCommand),
    /// Opérations longues.
    #[command(subcommand)]
    Operation(OperationCommand),
    /// Boîtes d'un compte.
    #[command(subcommand)]
    Mailbox(MailboxCommand),
    /// Lecture des messages, de leur corps et de leurs pièces jointes.
    #[command(subcommand)]
    Message(MessageCommand),
    /// Fils de conversation.
    #[command(subcommand)]
    Thread(ThreadCommand),
    /// Recherche locale et sur le serveur.
    #[command(subcommand)]
    Search(SearchCommand),
    /// Maintenance du profil : volumes, caches, index, exports.
    #[command(subcommand)]
    Maintenance(MaintenanceCommand),
    /// Actions sur les messages (marquer, déplacer, archiver, supprimer).
    #[command(subcommand)]
    Action(ActionCommand),
    /// Rédaction et envoi de messages.
    #[command(subcommand)]
    Draft(DraftCommand),
    /// File d'actions locale.
    #[command(subcommand)]
    Queue(QueueCommand),
    /// Lecture des événements du moteur.
    Events(EventsArgs),
}
