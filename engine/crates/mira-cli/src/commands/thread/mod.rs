//! Commandes `mira thread …` : fils de conversation d'une boîte (lecture locale).

use clap::{Args, Subcommand};
use mira_assembly::api::{Request, ThreadRequest};
use mira_assembly::mira_vocabulary::ThreadId;

use super::paging::PageArgs;
use super::targets::MailboxArg;

/// Fil visé par une commande.
#[derive(Debug, Args)]
pub struct ThreadArg {
    /// Identifiant du fil.
    pub thread: String,
}

/// Options de `mira thread list`.
#[derive(Debug, Args)]
pub struct ListArgs {
    #[command(flatten)]
    pub mailbox: MailboxArg,
    #[command(flatten)]
    pub page: PageArgs,
}

/// Commandes du domaine `thread`.
#[derive(Debug, Subcommand)]
pub enum ThreadCommand {
    /// Liste les fils d'une boîte, du plus récent au plus ancien.
    List(ListArgs),
    /// Affiche un fil et ses messages.
    Get(ThreadArg),
}

/// Traduit une commande `thread` en requête du contrat.
pub fn build_request(command: &ThreadCommand) -> Request {
    let request = match command {
        ThreadCommand::List(args) => ThreadRequest::ListThreads {
            mailbox: args.mailbox.id(),
            page: args.page.page_request(),
        },
        ThreadCommand::Get(target) => ThreadRequest::GetThread {
            thread: ThreadId::from_raw(target.thread.as_str()),
        },
    };
    Request::Thread(request)
}
