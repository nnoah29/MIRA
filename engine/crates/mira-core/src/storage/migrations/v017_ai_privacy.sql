-- Confidentialité de l'IA : types de contenu autorisés et réglage du masquage, par connexion.
ALTER TABLE ai_connections ADD COLUMN allowed_kinds TEXT NOT NULL DEFAULT '[]';
ALTER TABLE ai_connections ADD COLUMN masking_json TEXT NOT NULL DEFAULT '{"mode":"mask","categories":["iban","card","secret","otp"]}';

-- Une connexion qui avait consenti garde le comportement d'avant : le mail ouvert seulement.
UPDATE ai_connections SET allowed_kinds = '["mail_message"]' WHERE consent_at IS NOT NULL;
