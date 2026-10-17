//! Messages HTML avec image distante et lecture de leur HTML sûr par la façade.

use mira_core::api::{MessageRequest, Request, Response};
use mira_core::html::SafeHtml;
use mira_vocabulary::MessageId;

use super::calls::run;
use super::seed::add_body;
use super::world::World;

pub const REMOTE_HTML: &str = "<p>Bonjour</p><img src=\"https://tracker.example/pixel.gif\">";

/// Deux messages du même expéditeur, tous deux avec une image distante
pub fn seed_remote_pair(world: &World) {
    world.seed_messages(2);
    for id in ["m1", "m2"] {
        add_body(
            &world.mirror,
            &MessageId::from_raw(id),
            "texte",
            Some(REMOTE_HTML),
        );
    }
}

/// HTML sûr d'un message, rendu par la façade
pub async fn safe_html(world: &World, id: &str) -> SafeHtml {
    let request = Request::Message(MessageRequest::GetSafeHtml {
        message: MessageId::from_raw(id),
    });
    match run(&world.fixture.engine, request).await.unwrap() {
        Response::SafeHtml(safe) => safe,
        other => panic!("réponse inattendue : {other:?}"),
    }
}
