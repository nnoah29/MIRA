//! Nettoyage léger du HTML d'origine : garde la mise en forme, retire tout élément actif.

use std::borrow::Cow;
use std::collections::HashSet;

use super::remote::targets_private_machine;

const LAYOUT_ATTRIBUTES: [&str; 14] = [
    "style",
    "class",
    "id",
    "align",
    "valign",
    "bgcolor",
    "width",
    "height",
    "border",
    "cellpadding",
    "cellspacing",
    "color",
    "face",
    "size",
];

/// Nettoie le HTML d'origine : styles et mise en page gardés, scripts, cadres, formulaires et redirections retirés
pub fn sanitize_original_html(input: &str) -> String {
    let mut builder = ammonia::Builder::default();
    builder
        .url_schemes(HashSet::from(["http", "https", "mailto", "cid", "data"]))
        .url_relative(ammonia::UrlRelative::PassThrough)
        .link_rel(Some("noopener noreferrer nofollow"))
        .add_tags(["style", "center", "font"])
        .clean_content_tags(HashSet::from(["script"]))
        .add_generic_attributes(LAYOUT_ATTRIBUTES)
        .attribute_filter(|element, attribute, value| {
            (!targets_private_machine(element, attribute, value)).then_some(Cow::Borrowed(value))
        });
    builder.clean(input).to_string()
}
