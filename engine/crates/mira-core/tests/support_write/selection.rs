//! Application d'une action au résultat d'une requête de liste.

use mira_core::api::{ActionRequest, Request, Response, TargetSelection, WriteResponse};
use mira_core::reading::ListQuery;
use mira_vocabulary::{ActionId, ActionKind, StandardView, ViewRef};

use super::builders::key;
use super::fixture::Fixture;

impl Fixture {
    /// Applique l'action à tous les messages de la vue standard et rend les actions mises en file
    pub async fn apply_to_view(
        &self,
        kind: ActionKind,
        view: StandardView,
        key_text: &str,
    ) -> Vec<ActionId> {
        let query = ListQuery::of(ViewRef::Standard { view });
        let request = Request::Action(ActionRequest::ApplyAction {
            kind,
            targets: TargetSelection::Query { query },
            idempotency_key: key(key_text),
        });
        match self.run(request).await {
            Response::Write(WriteResponse::Actions(actions)) => actions,
            other => panic!("réponse inattendue : {other:?}"),
        }
    }
}
