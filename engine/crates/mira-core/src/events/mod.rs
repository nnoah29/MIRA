//! Bus d'événements numérotés (12.5) : chaque événement est écrit en base, puis diffusé en direct.
//! Un abonné rattrape d'abord ce qu'il a manqué depuis la base, puis reçoit le direct (invariant 32).

mod alerts;
mod bus;
mod counters;
mod recording;
mod sink;
mod subscription;

pub use alerts::{alert_for, announce_alert};
pub use bus::EventBus;
pub use recording::RecordingSink;
pub use sink::EventSink;
pub use subscription::EventSubscription;
