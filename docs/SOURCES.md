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
