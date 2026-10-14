//! Lecture d'un fichier local donné par le client comme pièce jointe.

use std::fs::File;
use std::io::Read;
use std::path::Path;

use mira_vocabulary::error::codes::local_storage;
use mira_vocabulary::{ErrorCategory, MiraError};

use super::mime_type::mime_type_for;
use crate::actions::errors::input::invalid_parameter;

/// Taille maximale d'une pièce jointe, en octets (25 Mo).
pub const MAX_ATTACHMENT_BYTES: u64 = 25 * 1024 * 1024;

/// Fichier lu : nom sans chemin, type MIME deviné, contenu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct LocalFile {
    pub(super) filename: String,
    pub(super) mime_type: String,
    pub(super) data: Vec<u8>,
}

/// Lit le fichier si c'est un fichier ordinaire de 25 Mo au plus (sinon `invalid_input.parameter`)
pub(super) fn read_local_file(path: &Path) -> Result<LocalFile, MiraError> {
    let announced = std::fs::metadata(path).map_err(|_| unreadable())?;
    if !announced.is_file() {
        return Err(invalid_parameter("la pièce jointe n'est pas un fichier"));
    }
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| invalid_parameter("nom de fichier inexploitable"))?
        .to_string();
    let file = File::open(path).map_err(|_| unreadable())?;
    let data = read_bounded(file)?;
    Ok(LocalFile {
        mime_type: mime_type_for(&filename).to_string(),
        filename,
        data,
    })
}

/// Lit le fichier ouvert (une seule ouverture), en s'arrêtant un octet après la limite
fn read_bounded(file: File) -> Result<Vec<u8>, MiraError> {
    let opened = file.metadata().map_err(|_| unreadable())?;
    if !opened.is_file() {
        return Err(invalid_parameter("la pièce jointe n'est pas un fichier"));
    }
    if opened.len() > MAX_ATTACHMENT_BYTES {
        return Err(too_large());
    }
    let mut data = Vec::new();
    file.take(MAX_ATTACHMENT_BYTES + 1)
        .read_to_end(&mut data)
        .map_err(|_| unreadable())?;
    if u64::try_from(data.len()).unwrap_or(u64::MAX) > MAX_ATTACHMENT_BYTES {
        return Err(too_large());
    }
    Ok(data)
}

/// Erreur « fichier illisible » (le chemin n'est pas repris dans le message)
fn unreadable() -> MiraError {
    MiraError::new(
        ErrorCategory::LocalStorage,
        local_storage::IO,
        "le fichier à joindre est introuvable ou illisible",
    )
}

/// Erreur « pièce jointe trop grosse »
fn too_large() -> MiraError {
    invalid_parameter("la pièce jointe dépasse 25 Mo")
}
