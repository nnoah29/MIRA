//! Exemples du domaine « libellés locaux » : libellés, poses, reprise.

use mira_core::api::{
    ImportReport, ImportedLabel, LocalLabelRequest, LocalLabelResponse, Request, Response,
};
use mira_vocabulary::{IdempotencyKey, LocalLabel, LocalLabelAssignment, LocalLabelId, ThreadId};

use super::{Named, numbered};

/// Libellé d'exemple
fn sample_label() -> LocalLabel {
    LocalLabel {
        id: LocalLabelId::from_raw("label-1"),
        name: "À payer".to_string(),
        color: 0xFFE8_542B,
    }
}

/// Les requêtes d'écriture : création, renommage, couleur, suppression
fn label_requests() -> Vec<LocalLabelRequest> {
    let id = LocalLabelId::from_raw("label-1");
    vec![
        LocalLabelRequest::CreateLocalLabel {
            name: "À payer".to_string(),
            color: Some(0xFFE8_542B),
            idempotency_key: IdempotencyKey("key-1".to_string()),
        },
        LocalLabelRequest::RenameLocalLabel {
            label: id.clone(),
            name: "Urgent".to_string(),
        },
        LocalLabelRequest::RecolorLocalLabel {
            label: id.clone(),
            color: 0xFF6A_A8D9,
        },
        LocalLabelRequest::DeleteLocalLabel { label: id },
    ]
}

/// Les requêtes de pose et de reprise
fn assignment_requests() -> Vec<LocalLabelRequest> {
    let key = || IdempotencyKey("key-1".to_string());
    vec![
        LocalLabelRequest::SetLocalLabel {
            label: LocalLabelId::from_raw("label-1"),
            threads: vec![ThreadId::from_raw("thread-1")],
            on: true,
            idempotency_key: key(),
        },
        LocalLabelRequest::ImportLocalLabels {
            labels: vec![ImportedLabel {
                name: "À payer".to_string(),
                color: 0xFFE8_542B,
                conversations: vec!["thread-1".to_string()],
            }],
            idempotency_key: key(),
        },
    ]
}

/// Une requête par opération de `LocalLabelRequest`
pub fn requests() -> Vec<Named<Request>> {
    let mut items = vec![LocalLabelRequest::ListLocalLabels];
    items.extend(label_requests());
    items.push(LocalLabelRequest::ListLocalLabelAssignments);
    items.extend(assignment_requests());
    items.extend(super::label_portability::requests());
    let ops: Vec<&str> = items.iter().map(op_name).collect();
    numbered("local_label", &ops)
        .into_iter()
        .zip(items.into_iter().map(Request::LocalLabel))
        .collect()
}

/// Une réponse par forme de `LocalLabelResponse`
pub fn responses() -> Vec<Named<Response>> {
    let assignment = LocalLabelAssignment {
        thread: ThreadId::from_raw("thread-1"),
        labels: vec![LocalLabelId::from_raw("label-1")],
    };
    #[rustfmt::skip]
    let report = ImportReport { labels: 1, assignments: 1, ignored: 0 };
    let items = [
        (
            "local_label.label",
            LocalLabelResponse::Label(sample_label()),
        ),
        (
            "local_label.labels",
            LocalLabelResponse::Labels(vec![sample_label()]),
        ),
        (
            "local_label.assignments",
            LocalLabelResponse::Assignments(vec![assignment]),
        ),
        ("local_label.imported", LocalLabelResponse::Imported(report)),
        ("local_label.done", LocalLabelResponse::Done),
    ];
    let wrap =
        |(name, inner): (&str, LocalLabelResponse)| (name.to_string(), Response::LocalLabel(inner));
    let extra = super::label_portability::responses();
    items.into_iter().map(wrap).chain(extra).collect()
}

/// Nom de l'opération, par `match` exhaustif
pub fn op_name(request: &LocalLabelRequest) -> &'static str {
    match request {
        LocalLabelRequest::ListLocalLabels => "list_local_labels",
        LocalLabelRequest::CreateLocalLabel { .. } => "create_local_label",
        LocalLabelRequest::RenameLocalLabel { .. } => "rename_local_label",
        LocalLabelRequest::RecolorLocalLabel { .. } => "recolor_local_label",
        LocalLabelRequest::DeleteLocalLabel { .. } => "delete_local_label",
        LocalLabelRequest::SetLocalLabel { .. } => "set_local_label",
        LocalLabelRequest::ListLocalLabelAssignments => "list_local_label_assignments",
        LocalLabelRequest::ImportLocalLabels { .. } => "import_local_labels",
        LocalLabelRequest::ExportLocalLabelsFile => "export_local_labels_file",
        LocalLabelRequest::ImportLocalLabelsFile { .. } => "import_local_labels_file",
    }
}
