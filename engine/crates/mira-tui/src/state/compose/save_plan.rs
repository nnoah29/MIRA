//! Préparation de l'enregistrement : changements des seuls champs modifiés, adresses invalides écartées.

use mira_assembly::api::DraftChanges;
use mira_assembly::mira_vocabulary::AddressEntry;

use super::addresses::parse_addresses;
use super::field::{Field, FieldKind};
use super::state::ComposeState;

/// Ce qu'un enregistrement enverrait : les changements, le texte envoyé de chaque champ, les saisies invalides.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SavePlan {
    pub changes: DraftChanges,
    pub sent: Vec<(Field, String)>,
    pub invalid: Vec<(Field, String)>,
}

/// Place les adresses dans le champ de changements qui correspond.
fn set_entries(changes: &mut DraftChanges, field: Field, entries: Vec<AddressEntry>) {
    match field {
        Field::To => changes.to = Some(entries),
        Field::Cc => changes.cc = Some(entries),
        Field::Bcc => changes.bcc = Some(entries),
        Field::Subject | Field::Body => {}
    }
}

impl ComposeState {
    /// Plan d'enregistrement des champs modifiés depuis le dernier enregistrement.
    pub fn plan_save(&self) -> SavePlan {
        let mut plan = SavePlan::default();
        for field in Field::ALL.into_iter().filter(|f| self.is_dirty(*f)) {
            self.plan_field(field, &mut plan);
        }
        plan
    }

    /// Ajoute ce champ modifié au plan, ou le range parmi les invalides.
    fn plan_field(&self, field: Field, plan: &mut SavePlan) {
        let text = self.text(field);
        match field.kind() {
            FieldKind::Address => match parse_addresses(&text) {
                Ok(entries) => set_entries(&mut plan.changes, field, entries),
                Err(segment) => {
                    plan.invalid.push((field, segment));
                    return;
                }
            },
            FieldKind::Subject => plan.changes.subject = Some(text.clone()),
            FieldKind::Body => {
                plan.changes.body_text = Some(text.clone());
                plan.changes.body_html = self.had_html.then(String::new);
            }
        }
        plan.sent.push((field, text));
    }
}
