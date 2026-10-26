-- Brouillons de réponse créés par les règles : un par (message, rang), pour qu'un rejeu après arrêt brutal n'en crée pas un second (invariant 18).
CREATE TABLE rule_reply_drafts (
    message_id TEXT NOT NULL,
    rank INTEGER NOT NULL,
    draft_id TEXT NOT NULL,
    PRIMARY KEY (message_id, rank)
);
