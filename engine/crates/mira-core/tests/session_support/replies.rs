//! File de réponses d'un faux serveur : la dernière servie se répète quand la file est vide.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub type Reply = (u16, String);
pub type Replies = Arc<Mutex<(VecDeque<Reply>, Option<Reply>)>>;

/// Prochaine réponse en file ; la dernière servie se répète quand la file est vide
pub fn next_reply(replies: &Replies) -> Reply {
    let mut state = replies.lock().unwrap();
    if let Some(next) = state.0.pop_front() {
        state.1 = Some(next);
    }
    state.1.clone().unwrap_or((500, "{}".to_string()))
}
