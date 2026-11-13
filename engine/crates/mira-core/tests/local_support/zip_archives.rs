//! Fabrique des archives `.zip` pour les tests d'extraction.

use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};

use zip::ZipWriter;
use zip::write::SimpleFileOptions;

/// Une entrée à écrire : fichier ordinaire ou lien symbolique.
pub enum ZipEntry {
    File(String, Vec<u8>),
    Symlink(String, String),
}

impl ZipEntry {
    /// Un fichier ordinaire
    pub fn file(name: &str, data: &[u8]) -> Self {
        Self::File(name.to_string(), data.to_vec())
    }

    /// Un lien symbolique vers `target`
    pub fn symlink(name: &str, target: &str) -> Self {
        Self::Symlink(name.to_string(), target.to_string())
    }
}

/// Octets d'une archive `.zip` contenant ces entrées.
pub fn zip_bytes(entries: &[ZipEntry]) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default();
    for entry in entries {
        match entry {
            ZipEntry::File(name, data) => {
                writer.start_file(name, options).unwrap();
                writer.write_all(data).unwrap();
            }
            ZipEntry::Symlink(name, target) => {
                writer.add_symlink(name, target, options).unwrap();
            }
        }
    }
    writer.finish().unwrap().into_inner()
}

/// Écrit l'archive `.zip` dans `dir` et rend son chemin.
pub fn write_zip(dir: &Path, entries: &[ZipEntry]) -> PathBuf {
    let path = dir.join("server.zip");
    std::fs::write(&path, zip_bytes(entries)).unwrap();
    path
}
