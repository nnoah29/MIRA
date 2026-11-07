//! Icône de zone de notification (StatusNotifierItem) : trois entrées de menu et une ligne d'état.

mod item;
mod menu;
mod start;
mod status;

pub use start::{StatusNotifierTray, TrayControl, TrayError};
pub use status::{TrayStatus, status_line, tray_status_of};
