//! Rejeu de la file d'actions vers le serveur : ordre, erreurs classées, reprise après plantage (10.7, 10.8).

mod attempt;
mod backoff;
mod classify;
mod context;
pub(crate) mod errors;
mod execute;
mod job;
mod park;
mod recover;
mod report;
mod run;
mod settle;

pub use classify::send_outcome_is_unknown;
pub use context::{BoxFuture, ReplayContext, ReplayCredentials};
pub(crate) use execute::{effective, purge_selected};
pub use recover::{RecoveryReport, recover_interrupted_actions};
pub(crate) use report::StepOutcome;
pub use report::{ReplayReport, ReplayStop};
pub use run::replay_actions;
