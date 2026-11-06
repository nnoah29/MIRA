//! Écouteurs du démon : socket Unix (hors Windows) et TCP loopback sur demande, chacun servi par sa tâche.

use std::process::ExitCode;
use std::sync::Arc;

use tokio::task::JoinHandle;

use crate::config::{DaemonConfig, DaemonPaths};
use crate::report::{say, say_error};
use crate::server::{ServerContext, serve};
use crate::transport::tcp::LoopbackTcpListener;
#[cfg(unix)]
use crate::transport::unix::UnixSocketListener;

/// Lie les écouteurs et lance leur service ; une socket déjà occupée est un démon déjà là (code 3).
pub fn start(
    config: &DaemonConfig,
    paths: &DaemonPaths,
    ctx: &Arc<ServerContext>,
) -> Result<Vec<JoinHandle<()>>, ExitCode> {
    let mut tasks = Vec::new();
    #[cfg(unix)]
    match UnixSocketListener::bind(&paths.socket) {
        Ok(listener) => tasks.push(tokio::spawn(serve(listener, ctx.clone()))),
        Err(error) if error.kind() == std::io::ErrorKind::AddrInUse => {
            return Err(say_error("un démon écoute déjà sur cette socket", 3));
        }
        Err(_) => return Err(say_error("socket Unix inutilisable", 1)),
    }
    if let Some(port) = config.tcp_port {
        let listener =
            LoopbackTcpListener::bind(port).map_err(|_| say_error("port TCP inutilisable", 1))?;
        say(&format!("tcp_port={}", listener.local_port()));
        tasks.push(tokio::spawn(serve(listener, ctx.clone())));
    }
    Ok(tasks)
}
