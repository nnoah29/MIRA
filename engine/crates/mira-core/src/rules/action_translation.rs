//! Traduction des actions de règle en actions de file, en changements de libellés ou en indicateurs.

use super::outcome::{LabelChange, RuleOutcome};
use mira_vocabulary::{ActionKind, LocalLabelId, RuleAction};

/// Ajoute au résultat les effets d'une règle et met à jour les libellés courants ; vrai si elle demande l'arrêt.
pub fn apply_actions(
    actions: &[RuleAction],
    outcome: &mut RuleOutcome,
    labels: &mut Vec<LocalLabelId>,
) -> bool {
    let mut stop = false;
    for action in actions {
        match action {
            RuleAction::AddLocalLabel { label } => change_label(label, true, outcome, labels),
            RuleAction::RemoveLocalLabel { label } => change_label(label, false, outcome, labels),
            RuleAction::StopProcessing => stop = true,
            RuleAction::SuppressNotification => outcome.silenced = true,
            RuleAction::ReplyWithTemplate { template } => {
                outcome.reply_templates.push(template.clone());
            }
            other => outcome.actions.extend(translate(other)),
        }
    }
    stop
}

/// Note un changement de libellé et le répercute sur les libellés courants du message
fn change_label(
    label: &LocalLabelId,
    on: bool,
    outcome: &mut RuleOutcome,
    labels: &mut Vec<LocalLabelId>,
) {
    outcome.label_changes.push(LabelChange {
        label: label.clone(),
        on,
    });
    labels.retain(|known| known != label);
    if on {
        labels.push(label.clone());
    }
}

/// Action de file équivalente, ou rien pour les actions qui n'en sont pas une.
fn translate(action: &RuleAction) -> Option<ActionKind> {
    match action {
        RuleAction::MarkRead { read } => Some(ActionKind::MarkRead { read: *read }),
        RuleAction::MarkFlagged { flagged } => Some(ActionKind::MarkFlagged { flagged: *flagged }),
        RuleAction::MoveTo { mailbox } => Some(ActionKind::Move {
            to: mailbox.clone(),
        }),
        RuleAction::AddToMailbox { mailbox } => Some(ActionKind::AddToMailbox {
            mailbox: mailbox.clone(),
        }),
        RuleAction::RemoveFromMailbox { mailbox } => Some(ActionKind::RemoveFromMailbox {
            mailbox: mailbox.clone(),
        }),
        RuleAction::Archive => Some(ActionKind::Archive),
        RuleAction::Trash => Some(ActionKind::Trash),
        RuleAction::MarkSpam { spam } => Some(ActionKind::MarkSpam { spam: *spam }),
        RuleAction::AddLocalLabel { .. }
        | RuleAction::RemoveLocalLabel { .. }
        | RuleAction::SuppressNotification
        | RuleAction::ReplyWithTemplate { .. }
        | RuleAction::StopProcessing => None,
    }
}
