//! Rangement unique d'un corps analysé dans le miroir, partagé par la synchronisation et la lecture.

mod attachments;
mod store;

pub(crate) use store::store_parsed_body;
