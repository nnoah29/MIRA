//! Libellés locaux sous forme portable : nom, couleur et réglages en clair, sans identifiant ni pose.

mod color;
mod entry;
mod from_portable;
mod settings;
mod to_portable;

pub use color::{format_color, parse_color};
pub use entry::{read_portable_entry, to_portable_with_settings};
pub use from_portable::{PortableLabel, PortableLabelError, from_portable};
pub use settings::PortableSettings;
pub use to_portable::to_portable;
