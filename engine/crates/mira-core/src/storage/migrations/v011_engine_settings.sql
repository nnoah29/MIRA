-- Réglages du moteur (chantier réglages) : paires clé / valeur texte, une ligne par réglage renseigné.
CREATE TABLE engine_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
