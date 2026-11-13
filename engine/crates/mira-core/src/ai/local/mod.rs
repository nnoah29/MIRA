//! Modèle d'IA local optionnel : ce que le descripteur décrit est installé et lancé ici, sans nom de service.

mod connection;
mod download;
mod download_errors;
mod download_transfer;
mod download_verify;
mod endpoint;
mod extract;
mod extract_any;
mod extract_check;
mod extract_modes;
mod extract_zip;
mod file_mode;
mod install;
mod install_slot;
mod key_file;
mod launch;
mod launch_args;
mod model_capabilities;
mod os_process;
mod paths;
mod process;
mod running;
mod runtime;
mod runtime_state;
mod server_install;
mod status;
mod zip_names;

pub(crate) use connection::trim_local_connection;
pub use download::{CancelFlag, DownloadJob, download_file};
pub(crate) use endpoint::{installed_ids, installed_models, local_endpoint};
pub use extract::extract_tar_gz;
pub use extract_zip::extract_zip;
pub(crate) use install::{InstallPlan, run_install};
pub use install_slot::LocalInstallSlot;
pub use launch_args::{ArgValues, build_args, thread_count};
pub use model_capabilities::tools_allowed;
pub use paths::LocalPaths;
pub use runtime::{LocalRuntimeHandle, LocalRuntimeManager, RuntimeContext};
pub use server_install::{
    ServerInstall, current_platform, install_server, platform_of, select_build,
};
pub(crate) use status::{local_descriptor, local_status};
