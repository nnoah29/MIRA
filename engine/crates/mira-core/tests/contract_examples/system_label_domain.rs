//! Exemples du domaine « libellés système » : poses lues, choix manuel.

use mira_core::api::{Request, Response, SystemLabelRequest, SystemLabelResponse};
use mira_vocabulary::{SystemLabelAssignment, SystemLabelKind, ThreadId};

use super::{Named, numbered};

/// Une requête par opération de `SystemLabelRequest`
pub fn requests() -> Vec<Named<Request>> {
    let items = vec![
        SystemLabelRequest::ListSystemLabelAssignments,
        SystemLabelRequest::SetSystemLabel {
            kind: SystemLabelKind::Waiting,
            threads: vec![ThreadId::from_raw("thread-1")],
            on: true,
        },
    ];
    let ops: Vec<&str> = items.iter().map(op_name).collect();
    numbered("system_label", &ops)
        .into_iter()
        .zip(items.into_iter().map(Request::SystemLabel))
        .collect()
}

/// Une réponse par forme de `SystemLabelResponse`
pub fn responses() -> Vec<Named<Response>> {
    let assignment = SystemLabelAssignment {
        thread: ThreadId::from_raw("thread-1"),
        kinds: vec![SystemLabelKind::Waiting, SystemLabelKind::Mention],
    };
    vec![
        (
            "system_label.assignments".to_string(),
            Response::SystemLabel(SystemLabelResponse::Assignments(vec![assignment])),
        ),
        (
            "system_label.done".to_string(),
            Response::SystemLabel(SystemLabelResponse::Done),
        ),
    ]
}

/// Nom de l'opération, par `match` exhaustif
pub fn op_name(request: &SystemLabelRequest) -> &'static str {
    match request {
        SystemLabelRequest::ListSystemLabelAssignments => "list_system_label_assignments",
        SystemLabelRequest::SetSystemLabel { .. } => "set_system_label",
    }
}
