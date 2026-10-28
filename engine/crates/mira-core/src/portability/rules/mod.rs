//! Règles sous forme portable : identifiants locaux remplacés par adresses, chemins et noms.

mod from_portable;
mod import_refs;
mod refs;
mod stored;
mod to_portable;
mod walk;

pub use from_portable::{PortableError, from_portable};
pub use refs::{AccountRefs, LabelRefs, MailboxRefs, RuleRefs, TemplateRefs};
pub use stored::StoredRefs;
pub use to_portable::to_portable;
