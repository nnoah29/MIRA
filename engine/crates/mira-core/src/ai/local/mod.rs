//! Modèle d'IA local optionnel : ce que le descripteur décrit est installé et lancé ici, sans nom de service.

mod download;
mod download_errors;
mod download_transfer;
mod download_verify;
mod extract;
mod extract_check;
mod extract_modes;
mod model_capabilities;
mod server_install;

pub use download::{CancelFlag, DownloadJob, download_file};
pub use extract::extract_tar_gz;
pub use model_capabilities::tools_allowed;
pub use server_install::{ServerInstall, current_platform, install_server, select_build};
