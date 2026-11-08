//! Écouteurs du démon : socket Unix (hors Windows) et TCP loopback sur demande, chacun servi par sa tâche.

use std::process::ExitCode;
use std::sync::Arc;

use tokio::task::JoinHandle;

use crate::config::{DaemonConfig, DaemonPaths};
use crate::report::{say, say_error};
use crate::server::{ServerContext, serve};
use crate::transport::port_file::PortFile;
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
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
            return Err(say_error(
                "le dossier de la socket est ouvert à d'autres utilisateurs",
                1,
            ));
        }
        Err(_) => return Err(say_error("socket Unix inutilisable", 1)),
    }
    if let Some(port) = config.effective_tcp_port() {
        tasks.push(start_tcp(port, paths, ctx)?);
    }
    Ok(tasks)
}

/// Lie le port TCP loopback, publie son numéro dans `daemon.port` et lance le service ; un fichier tenu par un autre démon donne le code 3.
fn start_tcp(
    port: u16,
    paths: &DaemonPaths,
    ctx: &Arc<ServerContext>,
) -> Result<JoinHandle<()>, ExitCode> {
    let listener =
        LoopbackTcpListener::bind(port).map_err(|_| say_error("port TCP inutilisable", 1))?;
    let published = PortFile::publish(&paths.port, listener.local_port()).map_err(|error| {
        match error.kind() {
            std::io::ErrorKind::AddrInUse => say_error("un démon tourne déjà pour ce profil", 3),
            _ => say_error("fichier du port inutilisable", 1),
        }
    })?;
    say(&format!("tcp_port={}", listener.local_port()));
    let ctx = ctx.clone();
    Ok(tokio::spawn(async move {
        let _published = published;
        serve(listener, ctx).await;
    }))
}
