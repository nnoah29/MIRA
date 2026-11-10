//! Modèle d'IA local optionnel : ce que le descripteur décrit est installé et lancé ici, sans nom de service.

mod download;
mod download_errors;
mod download_transfer;
mod download_verify;
mod model_capabilities;

pub use download::{CancelFlag, DownloadJob, download_file};
pub use model_capabilities::tools_allowed;
