//! Modèle d'IA local optionnel : ce que le descripteur décrit est installé et lancé ici, sans nom de service.

mod download;
mod download_errors;
mod download_transfer;
mod download_verify;
mod endpoint;
mod extract;
mod extract_check;
mod extract_modes;
mod key_file;
mod launch;
mod launch_args;
mod model_capabilities;
mod paths;
mod process;
mod running;
mod runtime;
mod runtime_state;
mod server_install;

pub use download::{CancelFlag, DownloadJob, download_file};
pub(crate) use endpoint::{installed_models, local_endpoint};
pub use extract::extract_tar_gz;
pub use launch_args::{ArgValues, build_args, thread_count};
pub use model_capabilities::tools_allowed;
pub use paths::LocalPaths;
pub use runtime::{LocalRuntimeHandle, LocalRuntimeManager, RuntimeContext};
pub use server_install::{ServerInstall, current_platform, install_server, select_build};
