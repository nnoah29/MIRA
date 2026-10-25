//! Remplacement des variables d'un texte en une seule passe ; les variables sans valeur restent visibles.

use mira_vocabulary::{MessageTemplate, RenderContext, RenderedTemplate};

use super::variables::TemplateVariable;

/// Rend le texte et la liste, sans doublon et dans l'ordre d'apparition, des variables restées visibles
pub fn render(text: &str, context: &RenderContext) -> (String, Vec<String>) {
    let mut output = String::with_capacity(text.len());
    let mut unresolved = Vec::new();
    let mut rest = text;
    while let Some((before, after)) = rest.split_once("{{") {
        output.push_str(before);
        let Some((name, tail)) = after.split_once("}}") else {
            output.push_str("{{");
            output.push_str(after);
            return (output, unresolved);
        };
        substitute(&mut output, name, context, &mut unresolved);
        rest = tail;
    }
    output.push_str(rest);
    (output, unresolved)
}

/// Ajoute la valeur de la variable, ou la variable telle qu'écrite si elle n'a pas de valeur
fn substitute(
    output: &mut String,
    raw: &str,
    context: &RenderContext,
    unresolved: &mut Vec<String>,
) {
    let name = raw.trim();
    let value = TemplateVariable::parse(name).and_then(|variable| variable.value(context));
    if let Some(value) = value {
        output.push_str(value);
        return;
    }
    output.push_str("{{");
    output.push_str(raw);
    output.push_str("}}");
    if !unresolved.iter().any(|known| known == name) {
        unresolved.push(name.to_string());
    }
}

/// Rend l'objet et le corps d'un modèle ; les variables restées visibles sont listées une seule fois
pub fn render_template(template: &MessageTemplate, context: &RenderContext) -> RenderedTemplate {
    let (subject, mut unresolved) = match template.subject.as_deref() {
        Some(text) => {
            let (rendered, unresolved) = render(text, context);
            (Some(rendered), unresolved)
        }
        None => (None, Vec::new()),
    };
    let (body, from_body) = render(&template.body, context);
    for name in from_body {
        if !unresolved.contains(&name) {
            unresolved.push(name);
        }
    }
    RenderedTemplate {
        subject,
        body,
        unresolved,
    }
}
