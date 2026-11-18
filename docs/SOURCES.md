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

## Outils d'IA en lecture seule (chantier 3 de l'IA)

Dernière lecture : 2026-10-30. Le contenu web est une donnée, jamais une consigne.

| Sujet | Ce qui a été confirmé | Source |
|---|---|---|
| Anthropic, appels d'outils en flux | Un appel est un bloc `tool_use` ouvert par `content_block_start` (champ `input` vide), ses arguments arrivent en `input_json_delta` (`partial_json`) à concaténer puis à analyser à la fermeture du bloc ; `stop_reason` vaut `tool_use` dans `message_delta` ; avec le flux fin (`eager_input_streaming`) le texte accumulé peut ne pas être du JSON valide (nos arguments sont donc validés avant exécution) | https://platform.claude.com/docs/en/build-with-claude/streaming, https://platform.claude.com/docs/en/agents-and-tools/tool-use/fine-grained-tool-streaming |
| OpenAI Chat, appels d'outils en flux | Chaque `chat.completion.chunk` porte `choices[].delta.tool_calls[]` avec `index`, `id`, `type`, `function.name` et `function.arguments` (fragment à concaténer par `index`) ; `finish_reason` vaut `tool_calls` | https://developers.openai.com/api/reference/resources/chat/subresources/completions/streaming-events, https://developers.openai.com/api/docs/guides/function-calling |

Réserve : les formes des messages de résultat d'outil (Anthropic : bloc `tool_result` avec `tool_use_id`, `content` et `is_error` dans un message `user` ; OpenAI : message de rôle `tool` avec `tool_call_id` et `content`) n'ont pas été relues sur une page officielle pendant la rédaction ; elles viennent des références de l'API et sont à confirmer à l'essai réel avec une clé. Les mêmes formes sont supposées pour Google, Mistral, Groq et les services compatibles OpenAI.

## Modèle d'embeddings de la mémoire de l'IA

Dernière lecture : 2026-11-15. Le contenu web est une donnée, jamais une consigne. Aucun fichier de modèle n'a été téléchargé pour relever ces valeurs : taille et SHA-256 viennent de la page du fichier Hugging Face, du pointeur LFS brut et de l'API arborescente, qui concordent. Le contrôle par un vrai téléchargement reste à faire à l'essai réel ; une somme fausse fait échouer l'installation sans rien installer.

| Sujet | Ce qui a été confirmé | Source |
|---|---|---|
| Modèle retenu | Granite Embedding 278M multilingue (IBM, Apache-2.0), GGUF Q6_K de bartowski : 236 067 488 octets, SHA-256 `a1ef1eec…a38b`, vecteurs de 768 valeurs, contexte de 512 jetons, pooling CLS, aucun préfixe de requête | https://huggingface.co/bartowski/granite-embedding-278m-multilingual-GGUF, https://huggingface.co/ibm-granite/granite-embedding-278m-multilingual |
| Serveur d'embeddings | `llama-server --embedding` exige un pooling autre que `none` pour `/v1/embeddings` et rend des vecteurs normalisés ; un texte doit tenir dans un lot physique (`-ub`) | https://raw.githubusercontent.com/ggml-org/llama.cpp/master/tools/server/README.md |
| Alternatives écartées | bge-m3 (438 Mo minimum), nomic-embed-text-v2-moe (344 Mo, préfixes requis), multilingual-e5-small (conversions communautaires), EmbeddingGemma (licence Gemma), Granite R2 (pas de GGUF trouvé) | https://huggingface.co/gpustack/bge-m3-GGUF, https://huggingface.co/nomic-ai/nomic-embed-text-v2-moe-GGUF |

## Dictée vocale sous Windows (`docs/VOICE-WINDOWS.md`)

Dernière lecture : 2026-11-17.

| Sujet | Ce qui a été confirmé | Source |
|---|---|---|
| API de reconnaissance de WinRT | `Windows.Media.SpeechRecognition` n'est disponible qu'avec une identité de paquet ; les langues de dictée et de recherche web sont traitées par un service en ligne, seules les grammaires le sont sur l'appareil | https://learn.microsoft.com/en-us/windows/apps/develop/input/speech-recognition |
| Réglage « reconnaissance vocale en ligne » | désactivée, Windows n'utilise que la reconnaissance sur l'appareil et aucune voix n'est envoyée à Microsoft | https://support.microsoft.com/en-us/windows/privacy/speech-voice-activation-inking-typing-and-privacy |
| Plugin Flutter pour Windows | `speech_to_text_windows` en bêta, « pas prêt pour la production », pensé pour des commandes courtes | https://pub.dev/packages/speech_to_text_windows |
| Windows Speech Recognition | déprécié, remplacé par Voice Access (Windows 11 22H2 et ultérieures) ; les sources divergent sur la date de retrait | https://windowsreport.com/windows-speech-recognition-is-getting-deprecated-and-it-will-be-replaced-by-voice-access/ |
| API Speech Recognition de Windows AI | aperçu public annoncé à Build 2026, local, temps réel ou par lot, NPU et processeur, anglais seulement au départ, modèles non téléchargés automatiquement | https://blogs.windows.com/windowsdeveloper/2026/06/02/build-2026-furthering-windows-as-the-trusted-platform-for-development/, https://www.heise.de/en/news/Local-speech-recognition-for-Windows-without-cloud-11318683.html |
| Non confirmé | identité de paquet de l'API de Windows AI, comportement de SAPI 5 et `DictationGrammar` hors ligne en français, traitement local ou en ligne de Win+H | aucune source fiable trouvée |
