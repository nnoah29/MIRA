//! Types publics du client IMAP : identifiants, drapeaux, résultats d'opérations.

mod flag;
mod login;
mod results;

pub use flag::ImapFlag;
pub use login::{ImapCredential, ImapLogin, ImapOptions};
pub use results::{FetchedFlags, FetchedHeaders, IdleOutcome, ResyncFrom, SelectedMailbox};
