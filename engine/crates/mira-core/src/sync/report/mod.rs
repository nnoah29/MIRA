//! Rapport d'activité d'un compte : passe en cours, dernier succès, dernière erreur, boîtes en cours.

mod handle;
mod mailbox;
mod pass;
mod state;

pub use handle::ReportHandle;
pub use state::SyncReport;
