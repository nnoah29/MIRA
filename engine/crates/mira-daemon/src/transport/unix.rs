//! Écouteur sur socket Unix : dossier 0700, socket 0600, contrôle de l'uid du pair.

use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

use tokio::net::UnixListener;

use super::unix_files::{clear_stale, prepare_directory};
use super::{Accepted, Listener, Trust};

/// Écoute sur une socket Unix privée ; le fichier est supprimé à la fermeture.
#[derive(Debug)]
pub struct UnixSocketListener {
    listener: UnixListener,
    path: PathBuf,
    owner_uid: u32,
}

/// Dit si le pair est le propriétaire de la socket.
pub fn peer_is_owner(peer_uid: u32, owner_uid: u32) -> bool {
    peer_uid == owner_uid
}

impl UnixSocketListener {
    /// Prépare le dossier, remplace une socket périmée, lie la socket et la passe en 0600.
    pub fn bind(path: &Path) -> std::io::Result<Self> {
        if let Some(directory) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
            prepare_directory(directory)?;
        }
        clear_stale(path)?;
        let listener = UnixListener::bind(path)?;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
        let owner_uid = std::fs::metadata(path)?.uid();
        Ok(Self {
            listener,
            path: path.to_path_buf(),
            owner_uid,
        })
    }
}

impl Listener for UnixSocketListener {
    /// Accepte la prochaine connexion d'un pair de même uid ; les autres sont fermées.
    async fn accept(&self) -> std::io::Result<Accepted> {
        loop {
            let (stream, _) = self.listener.accept().await?;
            let Ok(credentials) = stream.peer_cred() else {
                continue;
            };
            if peer_is_owner(credentials.uid(), self.owner_uid) {
                let trust = Trust::LocalUser(Some(credentials.uid()));
                return Ok(Accepted {
                    stream: Box::new(stream),
                    trust,
                });
            }
        }
    }

    /// `unix:<chemin>`
    fn describe(&self) -> String {
        format!("unix:{}", self.path.display())
    }
}

impl Drop for UnixSocketListener {
    /// Retire le fichier de socket
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
