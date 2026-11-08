//! Point d'entrée du démon MIRA : lit les options puis fait vivre le démon.
#![cfg_attr(windows, windows_subsystem = "windows")]

use std::process::ExitCode;

use clap::Parser;
use mira_daemon::config::DaemonConfig;

/// Lance le démon avec les options de la ligne de commande.
#[tokio::main]
async fn main() -> ExitCode {
    mira_daemon::run::run(DaemonConfig::parse()).await
}
