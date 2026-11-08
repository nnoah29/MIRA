//! Intégration au bureau Linux : notifications par D-Bus, relance de l'app au clic, langue du système.

mod attach;
#[cfg(target_os = "linux")]
mod connect;
#[cfg(not(target_os = "linux"))]
mod connect_other;
#[cfg(target_os = "linux")]
mod hints;
#[cfg(target_os = "linux")]
mod launch_app;
mod locale;
#[cfg(target_os = "linux")]
mod proxy;
#[cfg(target_os = "linux")]
mod sink;
#[cfg(target_os = "linux")]
mod state_mirror;
#[cfg(target_os = "linux")]
mod tray;

pub use attach::Desktop;
#[cfg(target_os = "linux")]
pub use hints::body_text;
#[cfg(target_os = "linux")]
pub use launch_app::{Launch, app_launcher, launch_plan};
pub use locale::{pick_locale, system_locale};
#[cfg(target_os = "linux")]
pub use sink::{DbusNotificationSink, DesktopError};
#[cfg(target_os = "linux")]
pub use state_mirror::mirror_state;
#[cfg(target_os = "linux")]
pub use tray::{
    StatusNotifierTray, TrayControl, TrayError, TrayStatus, status_line, tray_status_of,
};
