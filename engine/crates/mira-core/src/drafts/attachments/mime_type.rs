//! Type MIME d'une pièce jointe, deviné d'après l'extension du nom de fichier.

use std::path::Path;

const KNOWN_TYPES: [(&str, &str); 14] = [
    ("txt", "text/plain"),
    ("html", "text/html"),
    ("csv", "text/csv"),
    ("pdf", "application/pdf"),
    ("zip", "application/zip"),
    ("json", "application/json"),
    ("png", "image/png"),
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("gif", "image/gif"),
    ("webp", "image/webp"),
    ("svg", "image/svg+xml"),
    ("mp3", "audio/mpeg"),
    ("mp4", "video/mp4"),
];

/// Type MIME connu pour l'extension du nom, sinon `application/octet-stream`
pub(super) fn mime_type_for(filename: &str) -> &'static str {
    let extension = Path::new(filename)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_lowercase)
        .unwrap_or_default();
    KNOWN_TYPES
        .iter()
        .find(|(known, _)| *known == extension)
        .map_or("application/octet-stream", |(_, mime)| mime)
}
