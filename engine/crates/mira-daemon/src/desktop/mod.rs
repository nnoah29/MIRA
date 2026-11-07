//! Intégration au bureau Linux : notifications par D-Bus, relance de l'app au clic, langue du système.

mod attach;
mod hints;
mod launch_app;
mod locale;
mod proxy;
mod sink;

pub use attach::Desktop;
pub use launch_app::{Launch, app_launcher, launch_plan};
pub use locale::{pick_locale, system_locale};
pub use sink::{DbusNotificationSink, DesktopError};
