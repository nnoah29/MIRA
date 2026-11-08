//! Icône de zone de notification : StatusNotifierItem sous Linux, icône de la barre des tâches sous Windows.

mod control;
#[cfg(target_os = "linux")]
mod item;
#[cfg(target_os = "linux")]
mod menu;
#[cfg(target_os = "linux")]
mod start;
mod status;
#[cfg(windows)]
mod windows_tray;

pub use control::{TrayControl, TrayError};
#[cfg(target_os = "linux")]
pub use start::StatusNotifierTray;
pub use status::{TrayStatus, menu_labels, status_line, tray_status_of};
#[cfg(windows)]
pub use windows_tray::WindowsTray;
