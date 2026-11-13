#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]
//! Fabrique d'archives `.tar.gz` en mémoire, y compris piégées, pour les tests d'extraction.

use std::io::Write;

use flate2::Compression;
use flate2::write::GzEncoder;

/// Un bloc d'en-tête tar écrit à la main (le crate refuse les noms piégés)
fn raw_header(name: &str, size: usize, kind: u8, link: &str) -> [u8; 512] {
    let mut block = [0u8; 512];
    block[..name.len()].copy_from_slice(name.as_bytes());
    block[100..107].copy_from_slice(b"0000644");
    block[108..115].copy_from_slice(b"0000000");
    block[116..123].copy_from_slice(b"0000000");
    block[124..135].copy_from_slice(format!("{size:011o}").as_bytes());
    block[136..147].copy_from_slice(b"00000000000");
    block[156] = kind;
    block[157..157 + link.len()].copy_from_slice(link.as_bytes());
    block[257..263].copy_from_slice(b"ustar\0");
    block[148..156].copy_from_slice(b"        ");
    let sum: u32 = block.iter().map(|b| u32::from(*b)).sum();
    block[148..155].copy_from_slice(format!("{sum:06o}\0").as_bytes());
    block
}

/// Une entrée : fichier (`0`) ou lien symbolique (`2`) ; `data` vide pour un lien
pub struct Entry {
    pub name: String,
    pub kind: u8,
    pub link: String,
    pub data: Vec<u8>,
}

impl Entry {
    /// Fichier ordinaire
    pub fn file(name: &str, data: &[u8]) -> Self {
        Self {
            name: name.into(),
            kind: b'0',
            link: String::new(),
            data: data.to_vec(),
        }
    }

    /// Lien symbolique vers `target`
    pub fn symlink(name: &str, target: &str) -> Self {
        Self {
            name: name.into(),
            kind: b'2',
            link: target.into(),
            data: Vec::new(),
        }
    }
}

/// Assemble les entrées en `.tar.gz`
pub fn tar_gz(entries: &[Entry]) -> Vec<u8> {
    let mut tar = Vec::new();
    for entry in entries {
        tar.extend_from_slice(&raw_header(
            &entry.name,
            entry.data.len(),
            entry.kind,
            &entry.link,
        ));
        tar.extend_from_slice(&entry.data);
        tar.resize(tar.len().div_ceil(512) * 512, 0);
    }
    tar.extend_from_slice(&[0u8; 1024]);
    let mut gz = GzEncoder::new(Vec::new(), Compression::fast());
    gz.write_all(&tar).unwrap();
    gz.finish().unwrap()
}

/// Écrit l'archive dans `dir` sous `server.tar.gz` et rend son chemin
pub fn write_archive(dir: &std::path::Path, entries: &[Entry]) -> std::path::PathBuf {
    let path = dir.join("server.tar.gz");
    std::fs::write(&path, tar_gz(entries)).unwrap();
    path
}
