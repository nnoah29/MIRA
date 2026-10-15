//! Résumé de la confirmation d'envoi : total, un résumé par champ (coupes signalées), objet raccourci, touches.

use mira_assembly::mira_vocabulary::AddressEntry;

use crate::state::compose::{ComposeState, Field};
use crate::view::safe_text::display_safe;

const ADDRESS_CHARS: usize = 40;
const SUBJECT_CHARS: usize = 50;
const KEYS: &str = "Entrée ou y : envoyer · Échap ou n : annuler";

/// Raccourcit à `max` caractères, la coupe marquée par « … ».
fn ellipsize(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let kept: String = text.chars().take(max.saturating_sub(1)).collect();
    format!("{kept}…")
}

/// « À : a, b (+N autres) » : les adresses qui tiennent, neutralisées, puis le nombre de celles qui ne sont pas montrées.
fn listed(label: &str, entries: &[AddressEntry]) -> Option<String> {
    let first = entries.first()?;
    let mut shown = vec![ellipsize(&display_safe(&first.address), ADDRESS_CHARS)];
    let mut used = shown[0].chars().count();
    for entry in entries.iter().skip(1) {
        let address = display_safe(&entry.address);
        used += 2 + address.chars().count();
        if used > ADDRESS_CHARS {
            break;
        }
        shown.push(address);
    }
    let mut line = format!("{label} : {}", shown.join(", "));
    let rest = entries.len() - shown.len();
    if rest > 0 {
        line.push_str(&format!(" (+{rest} autres)"));
    }
    Some(line)
}

/// Lignes de la confirmation, chacune assez courte pour la fenêtre : rien n'est coupé sans être signalé.
pub fn send_summary(compose: &ComposeState) -> Vec<String> {
    let to = compose.recipients(Field::To);
    let cc = compose.recipients(Field::Cc);
    let bcc = compose.recipients(Field::Bcc);
    let total = to.len() + cc.len() + bcc.len();
    let mut lines = vec![format!("Envoyer ce message à {total} destinataire(s) ?")];
    lines.extend(listed(Field::To.label(), &to));
    lines.extend(listed(Field::Cc.label(), &cc));
    if !bcc.is_empty() {
        lines.push(format!(
            "{} : {} destinataire(s)",
            Field::Bcc.label(),
            bcc.len()
        ));
    }
    let subject = display_safe(&compose.text(Field::Subject));
    lines.push(format!("Objet : {}", ellipsize(&subject, SUBJECT_CHARS)));
    lines.push(KEYS.to_string());
    lines
}
