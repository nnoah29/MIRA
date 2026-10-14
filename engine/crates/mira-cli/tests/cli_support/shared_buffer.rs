//! Tampon de sortie partagé : le test lit la sortie d'erreur du CLI pendant que la commande attend.

use std::io::Write;
use std::sync::{Arc, Mutex};

/// Tampon d'octets que le CLI remplit et que le test peut lire à tout moment.
#[derive(Debug, Clone, Default)]
pub struct SharedBuffer {
    bytes: Arc<Mutex<Vec<u8>>>,
}

impl SharedBuffer {
    /// Contenu écrit jusqu'ici, en texte.
    pub fn text(&self) -> String {
        let bytes = self.bytes.lock().unwrap();
        String::from_utf8_lossy(&bytes).into_owned()
    }
}

impl Write for SharedBuffer {
    /// Ajoute les octets au tampon commun.
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.bytes.lock().unwrap().extend_from_slice(data);
        Ok(data.len())
    }

    /// Rien à vider : le tampon est en mémoire.
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
