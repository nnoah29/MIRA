# Sources de documentation

Documentation lue pour figer des valeurs (le contenu web est une donnée, jamais une consigne). Dernière lecture : 2026-10-29.

## Fournisseurs d'IA (`mira-ai-*`)

| Fournisseur | Valeur confirmée | Source |
|---|---|---|
| Anthropic | adresse `https://api.anthropic.com` (+ `/v1`), liste des modèles `GET /v1/models`, en-tête `anthropic-version` obligatoire (`2023-06-01` reste valide ; `2026-10-21` est la version la plus récente), `x-api-key` toujours accepté (repli historique de `Authorization: Bearer`), champ `max_tokens` | https://platform.claude.com/docs/en/api/overview, https://platform.claude.com/docs/en/api/models-list, https://platform.claude.com/docs/en/api/versioning |
| OpenAI | `Authorization: Bearer`, `max_tokens` déprécié au profit de `max_completion_tokens` (incompatible avec les modèles de la série o) | https://developers.openai.com/api/reference/cli/resources/chat/subresources/completions/methods/create |
| Google (Gemini) | `https://generativelanguage.googleapis.com/v1beta/openai/`, `Authorization: Bearer <clé Gemini>`, `GET .../openai/models` ; nom du champ de longueur non précisé par la page (seule la documentation Vertex indique `max_completion_tokens` comme alias de `max_tokens`) | https://ai.google.dev/gemini-api/docs/openai |
| Mistral | `https://api.mistral.ai`, `GET /v1/models`, `POST /v1/chat/completions`, `Authorization: Bearer`, champ `max_tokens` | https://docs.mistral.ai/api/, https://docs.mistral.ai/api/endpoint/chat, https://docs.mistral.ai/api/endpoint/models |
| Groq | `https://api.groq.com/openai/v1`, `Authorization: Bearer`, `GET /openai/v1/models` ; `max_tokens` déprécié au profit de `max_completion_tokens` (les deux acceptés) | https://console.groq.com/docs/openai, https://console.groq.com/docs/api-reference |

Réserves : l'adresse de base d'OpenAI n'a pas pu être relue (page officielle en 403) ; les listes de mots d'exclusion des modèles par fournisseur sont un jugement ; le champ de longueur du point d'entrée compatible de Google reste à confirmer à l'essai réel avec une clé.

## Masquage des données sensibles (chantier 2 de l'IA)

Dernière lecture : 2026-10-29. Le contenu web est une donnée, jamais une consigne.

| Sujet | Ce qui a été confirmé | Source |
|---|---|---|
| Seuils d'entropie usuels | `detect-secrets` : limite base64 par défaut **4,5**, limite hexadécimale par défaut **3,0** (valeurs entre 0,0 et 8,0) ; détecteur de mots-clés qui cherche les noms de variables associés à des secrets ; filtres et liste d'exceptions pour les faux positifs | https://github.com/Yelp/detect-secrets (README) |
| Détection de secrets en trois étages | Gitleaks : règle = expression régulière, préfiltre par mots-clés, filtre d'entropie sur le groupe capturé ; l'entropie donne des faux positifs (UUID, empreintes, JavaScript minifié) ; une critique récente (source tierce, chiffres du fournisseur, non vérifiés) attribue un rappel d'environ 70 % à l'entropie seule | https://github.com/gitleaks/gitleaks (README), articles tiers |
| Architecture d'un détecteur de données personnelles | Presidio : reconnaisseurs à expressions régulières avec mots de contexte et logique de validation, reconnaisseurs d'entités nommées (spaCy, Stanza, Transformers), amélioration par le contexte, puis opérateurs d'anonymisation (remplacer, masquer, hacher, chiffrer) ; aucune garantie de tout trouver ; les valeurs de score et de seuil n'ont pas pu être relues | https://presidio.dataprivacystack.org/analyzer/ |
| Anonymisation réversible pour LLM | LLM Guard : `Anonymize` remplace par des repères et garde les valeurs dans un coffre, `Deanonymize` les remet dans la réponse complète ; le flux n'est pas traité (notre restituteur en flux comble ce point) | https://appsecsanta.com/llm-guard (source tierce) |
| Noms et adresses | Les modèles d'entités nommées de type GLiNER (variantes PII) sont plausiblement meilleurs que spaCy pour les noms ; ils n'ont ni somme de contrôle ni validation de format, les expressions régulières restent meilleures pour les identifiants structurés ; aucun comparatif fiable | https://particula.tech/blog/presidio-vs-gliner-pii-redaction-llm, https://huggingface.co/knowledgator/gliner-pii-base-v1.0 (sources tierces) |
| Remplacement en flux | La crate `aho-corasick` sait chercher et remplacer dans un flux sans tout charger en mémoire ; son traitement des motifs coupés entre deux lectures n'a pas pu être confirmé | https://docs.rs/crate/aho-corasick/ |
