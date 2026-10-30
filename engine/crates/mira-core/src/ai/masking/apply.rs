//! Masqueur d'une requête : remplace les valeurs sensibles par des repères et garde la table en mémoire.

use std::collections::{HashMap, HashSet};

use mira_vocabulary::{MaskingMode, MaskingSettings, SensitiveCategory};

use super::detect::detect;
use super::placeholder::{existing, label};
use super::restore::Restorer;

/// Table cumulée d'une requête : repères créés et valeurs d'origine, jamais écrite nulle part.
#[derive(Default)]
pub struct Masker {
    pub(super) enabled: bool,
    pub(super) categories: Vec<SensitiveCategory>,
    pub(super) counters: HashMap<SensitiveCategory, u32>,
    pub(super) reserved: HashSet<String>,
    pub(super) table: Vec<(String, String)>,
    pub(super) taken: usize,
}

impl Masker {
    /// Prépare un masqueur vide ; avec le mode coupé, `mask` rend le texte tel quel.
    pub fn new(settings: &MaskingSettings) -> Self {
        Self {
            enabled: settings.mode == MaskingMode::Mask,
            categories: settings.categories.clone(),
            ..Self::default()
        }
    }

    /// Note les repères déjà écrits dans ce texte pour ne jamais en attribuer le numéro.
    pub fn reserve(&mut self, text: &str) {
        self.reserved.extend(existing(text));
    }

    /// Remplace chaque valeur sensible du texte par un repère numéroté et retient la valeur.
    pub fn mask(&mut self, text: &str) -> String {
        if !self.enabled {
            return text.to_string();
        }
        self.reserve(text);
        let mut masked = String::with_capacity(text.len());
        let mut last = 0;
        for found in detect(text, &self.categories) {
            let placeholder = self.next_placeholder(found.category);
            masked.push_str(text.get(last..found.start).unwrap_or_default());
            masked.push_str(&placeholder);
            let value = text.get(found.start..found.end).unwrap_or_default();
            self.table.push((placeholder, value.to_string()));
            last = found.end;
        }
        masked.push_str(text.get(last..).unwrap_or_default());
        masked
    }

    /// Nombre de valeurs masquées depuis la création.
    pub fn count(&self) -> u32 {
        u32::try_from(self.table.len()).unwrap_or(u32::MAX)
    }

    /// Cède la table au restituteur de la réponse en flux.
    pub fn into_restorer(self) -> Restorer {
        Restorer::new(self.table)
    }

    /// Prochain repère libre de la catégorie, en sautant les numéros déjà présents dans les textes.
    fn next_placeholder(&mut self, category: SensitiveCategory) -> String {
        loop {
            let counter = self.counters.entry(category).or_insert(0);
            *counter += 1;
            let candidate = format!("[{}_{}]", label(category), counter);
            if !self.reserved.contains(&candidate) {
                return candidate;
            }
        }
    }
}
