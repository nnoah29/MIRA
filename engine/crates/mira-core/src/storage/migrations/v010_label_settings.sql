-- Réglages des libellés (R3), serveur ou locaux. Pas de clé étrangère (deux natures) : la suppression
-- d'une boîte ou d'un libellé local supprime sa ligne, et une ligne orpheline est ignorée à la lecture.
CREATE TABLE label_settings (
    kind TEXT NOT NULL,
    label_id TEXT NOT NULL,
    hidden INTEGER NOT NULL DEFAULT 0,
    hide_from_inbox INTEGER NOT NULL DEFAULT 0,
    notifications INTEGER NOT NULL DEFAULT 1,
    counter TEXT NOT NULL DEFAULT 'unread',
    color INTEGER,
    position INTEGER,
    PRIMARY KEY (kind, label_id)
);
