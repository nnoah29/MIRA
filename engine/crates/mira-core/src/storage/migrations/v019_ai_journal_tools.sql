-- Journal des envois d'IA : outils appelés pendant la génération (noms et nombres d'appels, jamais d'argument).
ALTER TABLE ai_journal ADD COLUMN tools_json TEXT NOT NULL DEFAULT '[]';
