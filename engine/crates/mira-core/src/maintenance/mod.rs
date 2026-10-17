//! Maintenance du profil : purge du cache, diagnostic et export, sans secret (12.4, D-29).

mod account_cache;
mod cache_purge;
mod diagnostic;
mod export_file;
mod profile_export;

pub use account_cache::discard_mirror_and_cache;
pub(crate) use account_cache::remove_account_cache_files;
pub use cache_purge::purge_attachment_cache;
pub use diagnostic::{DiagnosticReport, build_diagnostic};
pub use export_file::write_export_file;
pub use profile_export::{ProfileExport, build_profile_export};
