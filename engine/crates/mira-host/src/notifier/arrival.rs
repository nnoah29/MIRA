//! Un message arrivé, réduit à ce qu'il faut pour composer sa notification.

/// En-tête d'un message arrivé et compte qui le reçoit ; jamais le corps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Arrival {
    pub account: String,
    pub address: String,
    pub message: String,
    pub sender: String,
    pub subject: String,
}
