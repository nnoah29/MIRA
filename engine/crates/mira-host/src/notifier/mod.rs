//! Notifieur du moteur : décide quoi notifier et compose le texte ; l'affichage revient à l'hôte (récepteur).

mod arrival;
mod clock;
mod compose;
mod decision;
mod flush;
mod gather;
mod notification;
mod pending;
mod run;
mod texts;

pub use arrival::Arrival;
pub use clock::{LocalClock, SystemClock};
pub use compose::compose;
pub use decision::{Decision, decide, in_quiet_hours};
pub use notification::{Notification, NotificationSink};
pub use run::run_notifier;
pub use texts::Texts;
