//! Commande `mira events` : lecture des événements par numéro (clients sans abonnement).

use clap::Args;
use mira_assembly::api::{EventRequest, Request};
use mira_assembly::mira_vocabulary::ids::keys::EventSeq;
use mira_assembly::mira_vocabulary::{AccountId, EventFilter};

/// Options de lecture des événements.
#[derive(Debug, Args)]
pub struct EventsArgs {
    /// Ne rend que les événements dont le numéro est supérieur à celui-ci.
    #[arg(long)]
    pub after: Option<u64>,
    /// Nombre maximal d'événements rendus.
    #[arg(long, default_value_t = 100)]
    pub limit: u32,
    /// Ne rend que les événements de ces comptes (répétable).
    #[arg(long = "account")]
    pub accounts: Vec<String>,
}

/// Traduit `mira events` en requête du contrat.
pub fn build_request(args: &EventsArgs) -> Request {
    let filter = EventFilter {
        accounts: args.accounts.iter().map(AccountId::from_raw).collect(),
        categories: Vec::new(),
        after: args.after.map(EventSeq),
    };
    Request::Event(EventRequest::GetEvents {
        filter,
        limit: args.limit,
    })
}
