//! Intégration au bureau Linux : notifications par D-Bus, relance de l'app au clic, langue du système.

mod attach;
mod hints;
mod launch_app;
mod locale;
mod proxy;
mod sink;
mod state_mirror;
mod tray;

pub use attach::Desktop;
pub use hints::body_text;
pub use launch_app::{Launch, app_launcher, launch_plan};
pub use locale::{pick_locale, system_locale};
pub use sink::{DbusNotificationSink, DesktopError};
pub use state_mirror::mirror_state;
pub use tray::{
    StatusNotifierTray, TrayControl, TrayError, TrayStatus, status_line, tray_status_of,
};
