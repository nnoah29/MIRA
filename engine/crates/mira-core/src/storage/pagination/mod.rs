//! Pagination par curseur opaque, sans `OFFSET` (invariant 29).

mod cursor;
mod page;

pub(crate) use page::{cursor_bounds, finish_page};
