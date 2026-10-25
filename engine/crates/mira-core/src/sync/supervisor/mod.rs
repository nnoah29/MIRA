//! Supervision : une tâche par compte, budget de connexions, reconnexion, états du compte, arrêt propre.

mod account_task;
mod config;
mod connected;
mod control;
mod exit;
mod idle_start;
mod poll;
mod running;
mod runtime;
mod states;
mod sync_supervisor;
mod wake;

pub use config::SupervisorConfig;
pub use sync_supervisor::SyncSupervisor;
