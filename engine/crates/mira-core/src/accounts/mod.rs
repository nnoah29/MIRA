//! Cycle de vie des comptes et des identités : recherche, choix du provider, serveurs, construction.

mod connection_check;
mod lookup;
mod network_policy;
mod new_account;
mod provider_choice;
mod servers;
mod signature;
mod state_change;

pub use connection_check::{check_imap_login, imap_login_of};
pub use lookup::{require_account, require_active_account, require_identity};
pub use network_policy::NetworkPolicy;
pub use new_account::{NewAccountParts, build_account, build_default_identity};
pub use provider_choice::{choose_provider, require_auth_method};
pub use servers::{ResolvedServers, resolve_servers};
pub use signature::normalize_signature;
pub use state_change::change_account_state;
