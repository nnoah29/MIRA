# Audit de sécurité MIRA — checklist OWASP

Branche `feat/SEC-01-audit-securite` (depuis `dev` @ 947f4af). Référentiels : OWASP Top 10:2021, OWASP Top 10 for LLM Applications, OWASP ASVS (sections applicables), OWASP MASVS (Android), chaîne d'approvisionnement.

## Légende

- ✅ **Couvert** : mécanisme présent, preuve citée (reste à confirmer par un test dédié si non indiqué).
- 🟡 **À vérifier** : mécanisme probable ou partiel, lecture de code approfondie nécessaire.
- ❌ **Non couvert** : lacune constatée.

> **Portée de cette première passe** : sondage statique ciblé (recherches par motifs, lecture des fichiers clés). Ce n'est pas une revue exhaustive ; chaque 🟡 est une tâche d'audit à instruire. Aucun test dynamique n'a été lancé.

## Surface d'attaque

| Surface | Entrées non fiables | Composants |
|---|---|---|
| Contenu des mails | HTML, MIME, pièces jointes, en-têtes, liens | `mira-core/{mime,html}`, `app/lib/ui/{web_mail_body,html_body}.dart` |
| Serveurs distants | réponses IMAP/SMTP/OAuth | `mira-core/{imap,smtp,auth}` |
| IPC local | trames du démon (socket Unix / TCP 127.0.0.1) | `mira-daemon`, `mira-protocol`, `mira-host` |
| Connecteurs IA | réponses de modèles, URL de base éditable, outils | `mira-ai-*`, `mira-core/ai` |
| Fichiers importés | règles YAML, étiquettes, archives, modèles locaux | `mira-core/{portability,rules,ai/local}` |
| Secrets | trousseau, jetons OAuth, clés IA | `mira-core/{ports/secrets,auth}`, `mira-daemon/token` |
| Distribution | paquets, installeur, identifiants OAuth embarqués | `packaging/`, `mira-assembly` |

## A01 — Contrôle d'accès défaillant

| ID | Contrôle | Statut | Preuve / à faire |
|---|---|---|---|
| SEC-A01-1 | Socket Unix en 0600, dossier en 0700 | ✅ | `transport/unix.rs:40`, `unix_files.rs:11-13` |
| SEC-A01-2 | Jeton local en fichier privé, refus si 0o077 | ✅ | `token/file.rs` `refuse_shared` |
| SEC-A01-3 | TCP lié à 127.0.0.1 seulement | ✅ | `transport/tcp.rs:18` |
| SEC-A01-4 | Comparaison du jeton en temps constant | ✅ | `token/mod.rs:23` : HMAC + `hmac::verify` (corrigé après lecture) |
| SEC-A01-5 | Contrôle du pair sur socket Unix (`SO_PEERCRED`, même UID) | 🟡 | non trouvé ; le 0600 couvre le cas courant |
| SEC-A01-6 | Sous TCP loopback : tout processus local de n'importe quel utilisateur peut tenter de se connecter | 🟡 | seul le jeton protège ; évaluer limitation de tentatives |
| SEC-A01-7 | Autorisations par opération (une session « lecture » ne peut pas écrire/envoyer) | 🟡 | `Auth` n'a qu'un `LocalToken` ; pas de notion de portée |
| SEC-A01-8 | Isolation entre comptes/profils (accès croisé par identifiant) | 🟡 | `profile/paths.rs` valide le nom de profil ; auditer les handlers par `account_id` |
| SEC-A01-9 | Opérations d'hôte (`Shutdown`, `RetryUnlock`) hors contrat d'API | ✅ | `protocol/auth.rs` `HostOp` |

## A02 — Défaillances cryptographiques

| ID | Contrôle | Statut | Preuve / à faire |
|---|---|---|---|
| SEC-A02-1 | TLS rustls, pas de `danger_accept_invalid_*` | ✅ | grep négatif ; `imap/connection/tls.rs` |
| SEC-A02-2 | Refus du clair (IMAP/SMTP/OAuth) | ✅ | `imap/connection/policy.rs`, `auth/loopback.rs:17` |
| SEC-A02-3 | Version TLS minimale (TLS 1.2+ explicite) | 🟡 | feature `tls12` activée ; confirmer qu'aucun TLS < 1.2 |
| SEC-A02-4 | STARTTLS : protection contre le *downgrade* / injection avant TLS | 🟡 | auditer `imap/connection/starttls.rs` et SMTP |
| SEC-A02-5 | Stockage des secrets dans le trousseau système | ✅ | `ports/secrets/keyring_store.rs` |
| SEC-A02-6 | Repli si trousseau indisponible : pas de secret en clair sur disque | 🟡 | `file_secrets/` dans `mira-host` ; auditer chiffrement et permissions |
| SEC-A02-7 | Base SQLite et corps de messages chiffrés au repos | ❌ | non constaté ; décision produit à documenter (risque : vol d'appareil) |
| SEC-A02-8 | Identifiants OAuth embarqués : clé AES stockée à côté du chiffré | ❌ | `oauth_embedded.rs` : simple obfuscation, `client_secret` extractable du binaire. Acceptable pour client public + PKCE, à documenter et à ne pas présenter comme un secret |
| SEC-A02-9 | PKCE S256 + `state` aléatoire + redirection loopback | ✅ | `auth/pkce.rs`, `flow/start_key.rs` |
| SEC-A02-10 | Aléa cryptographique pour jetons | ✅ | `random_hex` ; confirmer la source (`getrandom`/`ring`) |
| SEC-A02-11 | Effacement mémoire des secrets (`zeroize`) | 🟡 | à vérifier dans `secrets` et clés IA |

## A03 — Injection

| ID | Contrôle | Statut | Preuve / à faire |
|---|---|---|---|
| SEC-A03-1 | SQL paramétré | ✅ | `format!` SQL ne porte que des constantes (`COLUMNS`, nom de table) ; `?1` partout. À confirmer pour `introspection.rs:39` (nom de table) |
| SEC-A03-2 | Injection de commandes IMAP (noms de boîtes, critères de recherche) | ✅ | `quote`/`quote_mailbox` refusent CR/LF/NUL partout ; drapeaux en énumération fixe (lecture, 2026-11-26) |
| SEC-A03-3 | Injection CRLF dans en-têtes SMTP/MIME (From, Subject, Reply-To, noms de fichiers) | ✅ | `smtp/compose/sanitize.rs` ; tests `smtp_compose_validation`, `smtp_compose_attachment_injection` |
| SEC-A03-4 | XSS dans le rendu HTML (ammonia, schémas, `data:`, SVG, CSS) | 🟡 | `html/sanitize.rs` : schémas `http,https,mailto,cid,data` ; `data:` image hors SVG seulement. Tester corpus XSS, `<style>`, `srcset`, `<base>`, `<meta refresh>`, `<form>` |
| SEC-A03-5 | WebView (Linux WebKitGTK / Android) : JavaScript désactivé, navigation et sous-ressources bloquées | 🟡 | `web_mail_body.dart` : **aucun réglage de sécurité trouvé** (JS, `NavigationDelegate`, accès fichiers). À instruire en priorité |
| SEC-A03-6 | Ouverture de liens : liste blanche de schémas | ✅ | `html_body.dart` `openLink` (http/https/mailto) ; `Process.run` sans shell |
| SEC-A03-7 | Hameçonnage par liens (texte ≠ cible, homographes IDN) | ❌ | aucune alerte constatée |
| SEC-A03-8 | Injection de prompt via contenu de mail (IA) | 🟡 | voir section LLM |
| SEC-A03-9 | Injection YAML des règles importées | ✅ | alias, doublons, taille, éléments, imbrication : `portability_envelope_*` |

## A04 — Conception non sécurisée

| ID | Contrôle | Statut | Preuve / à faire |
|---|---|---|---|
| SEC-A04-1 | Modèle de menaces écrit (STRIDE) | ✅ | `MODELE-DE-MENACES.md` |
| SEC-A04-2 | Limites de taille/temps/profondeur (MIME, littéraux IMAP, HTML, pièces jointes) | ✅ | MIME : `mime_structure_limits` ; YAML : `portability_envelope_limits`/`_depth` |
| SEC-A04-3 | Chargement distant bloqué par défaut (pixels espions, confidentialité) | ✅ | `html/remote.rs` `is_blocked_until_allowed` |
| SEC-A04-4 | Confirmation avant envoi/suppression irréversibles initiés par l'IA | ✅ | aucun outil d'envoi ni de suppression ; propositions validées par clic |
| SEC-A04-5 | Idempotence et reprise sur crash (envoi en double) | ✅ | invariants du projet |

## A05 — Mauvaise configuration de sécurité

| ID | Contrôle | Statut | Preuve / à faire |
|---|---|---|---|
| SEC-A05-1 | Permissions des fichiers de données, export, clés IA | ✅ | `profile/paths.rs` 0700, `export_file.rs` 0600, `key_file.rs` 0600 |
| SEC-A05-2 | Permissions Windows (ACL) pour jeton et données | ❌ | `refuse_shared` est un no-op hors Unix (« phase Windows ») |
| SEC-A05-3 | Durcissement des paquets (deb/rpm/AppImage : permissions, scripts post-install) | 🟡 | auditer `packaging/` |
| SEC-A05-4 | Android : `allowBackup`, `exported`, `usesCleartextTraffic`, capture d'écran | 🟡 | auditer `app/android` |
| SEC-A05-5 | Processus IA local : environnement vidé, E/S neutralisées | ✅ | `ai/local/process.rs` `env_clear`, `kill_on_drop` |
| SEC-A05-6 | Fichiers `SECURITY.md` et politique de divulgation | ✅ | `SECURITY.md` à la racine |

## A06 — Composants vulnérables et obsolètes

| ID | Contrôle | Statut | Preuve / à faire |
|---|---|---|---|
| SEC-A06-1 | `cargo-deny` : bans, licences, sources | ✅ | `ci.sh:40-42` |
| SEC-A06-2 | Avis de sécurité (RustSec) | ✅ | `cargo deny check advisories` + `.github/workflows/securite.yml` |
| SEC-A06-3 | Audit des dépendances Dart/Flutter (`pubspec.lock`) | ❌ | aucun outil constaté |
| SEC-A06-4 | Dépendances sensibles : `zip`, `tar`, `flate2`, `ammonia`, `reqwest`, `rustls`, `keyring` | 🟡 | suivre les avis ; épingler les versions |
| SEC-A06-5 | Mise à jour automatique des dépendances (Dependabot/Renovate) | ✅ | `.github/dependabot.yml` |
| SEC-A06-6 | SBOM publié avec les releases | ❌ | non constaté |

## A07 — Identification et authentification

| ID | Contrôle | Statut | Preuve / à faire |
|---|---|---|---|
| SEC-A07-1 | OAuth2 PKCE pour Gmail/Outlook | ✅ | `auth/` |
| SEC-A07-2 | Écouteur de rappel loopback : port éphémère, usage unique, `state` vérifié, délai | 🟡 | `flow/pending.rs` ; tester un second appel, un `state` erroné |
| SEC-A07-3 | Renouvellement/révocation des jetons, révocation à la suppression de compte | 🟡 | `auth/renewal.rs` ; vérifier la purge du trousseau |
| SEC-A07-4 | Mots de passe d'application (Yahoo, générique) : stockage et journalisation | 🟡 | auditer `auth/form.rs` |
| SEC-A07-5 | Verrouillage de l'application / déverrouillage trousseau | 🟡 | `unlock.rs`, `RetryUnlock` ; pas de verrou applicatif constaté |

## A08 — Intégrité des logiciels et des données

| ID | Contrôle | Statut | Preuve / à faire |
|---|---|---|---|
| SEC-A08-1 | Téléchargement de modèles/serveur IA : somme SHA-256 vérifiée | ✅ | `ai/local/download_verify.rs` ; confirmer que la somme n'est pas lue depuis la même source que le fichier |
| SEC-A08-2 | Extraction d'archives : *zip-slip*, liens symboliques, bombes de décompression | ✅ | `extract_check.rs`, `max_bytes` ; ajouter tests adverses |
| SEC-A08-3 | Signature des paquets et des releases | 🟡 | auditer `packaging/release.sh` |
| SEC-A08-4 | Mise à jour du logiciel (canal, signature) | 🟡 | aucun auto-update trouvé ; confirmer |
| SEC-A08-5 | Import/export de profil : intégrité, chemins, écrasement | 🟡 | `portability/` |
| SEC-A08-6 | Épinglage des actions CI et permissions minimales du workflow | 🟡 | `permissions:` présent ; vérifier épinglage par SHA |

## A09 — Journalisation et supervision

| ID | Contrôle | Statut | Preuve / à faire |
|---|---|---|---|
| SEC-A09-1 | Aucun secret dans journaux/erreurs | ✅ | `Debug` masqué de `Auth` ; test d'absence de secrets (règle projet) |
| SEC-A09-2 | Journaux `tracing` : pas d'adresses, objets, corps de mails ni URL avec jeton | 🟡 | `account/add.rs`, `send/execute.rs`, `oauth_embedded.rs` |
| SEC-A09-3 | Journal d'audit des événements sensibles (échecs d'authentification IPC, ajout de compte, envoi) | ❌ | non constaté |
| SEC-A09-4 | Rapports de plantage / télémétrie sans données personnelles | 🟡 | confirmer qu'il n'y en a pas |
| SEC-A09-5 | Sortie Flutter : `print`/`debugPrint` de données de mail | 🟡 | grep `print(` sans résultat ; vérifier en profondeur |

## A10 — SSRF et requêtes sortantes

| ID | Contrôle | Statut | Preuve / à faire |
|---|---|---|---|
| SEC-A10-1 | URL de base IA éditable (`base_url_editable`) : refus de `file:`, HTTP clair, adresses privées/métadonnées cloud | ✅ | `ai/base_url.rs` : refus de `file:`, clair hors boucle locale, identifiants ; LAN en HTTPS permis |
| SEC-A10-2 | Redirections HTTP (suivre, limiter, ne pas transmettre la clé inter-hôtes) | ✅ | `ai/transport/client.rs` : `Policy::none()` |
| SEC-A10-3 | Chargement d'images distantes : pas d'accès aux adresses locales | 🟡 | passe par WebView/moteur ; dépend de SEC-A03-5 |
| SEC-A10-4 | Hôtes IMAP/SMTP saisis par l'utilisateur : ports, adresses internes | 🟡 | auditer l'auto-configuration (autodiscover) |
| SEC-A10-5 | Fuite via requêtes DNS/préchargement de liens | 🟡 | WebView |

## OWASP Top 10 pour applications LLM (connecteurs IA)

| ID | Contrôle | Statut | Preuve / à faire |
|---|---|---|---|
| SEC-LLM01 | Injection de prompt indirecte par le contenu d'un mail | 🟡 | séparer clairement données/instructions ; tester un mail hostile |
| SEC-LLM02 | Sortie non assainie : rendu Markdown/HTML de la réponse du modèle | ✅ | `chat_markdown.dart` : gras, italique, code seulement ; ni lien ni image |
| SEC-LLM03 | Outils à agence excessive (envoi, suppression, règles, transfert) sans confirmation humaine | ✅ | catalogue sans outil d'envoi ; `propose_action` validée par l'utilisateur ; consentement par type de contenu |
| SEC-LLM04 | Exfiltration de données : le mail ordonne au modèle de transmettre du contenu par outil/lien/image | ✅ | pas d'image ni de lien rendus dans les réponses ; images de mail bloquées par défaut |
| SEC-LLM05 | Mémoire IA (`ai_memories`) empoisonnable par un mail | ✅ | `remember`/`forget_memory` sont des propositions : rien n'est écrit avant le clic |
| SEC-LLM06 | Envoi de contenu privé à un tiers : consentement par fournisseur, minimisation | 🟡 | auditer ce qui part vers les 9 fournisseurs |
| SEC-LLM07 | Clés d'API : stockage, journalisation, envoi seulement à l'hôte prévu | 🟡 | `key_file.rs`, transports |
| SEC-LLM08 | Déni de portefeuille / boucles d'outils : plafonds de jetons et d'itérations | 🟡 | `ai/chat/ending.rs` |
| SEC-LLM09 | Modèles locaux : provenance et intégrité | ✅ | voir SEC-A08-1 |

## Spécifique poste de travail et mobile

| ID | Contrôle | Statut | Preuve / à faire |
|---|---|---|---|
| SEC-D-1 | Pièces jointes : nom assaini, pas d'exécution automatique, avertissement sur types exécutables | 🟡 | `drafts/attachments/local_file.rs`, `fetching/atomic_write.rs` |
| SEC-D-2 | Écriture atomique et suivi des liens symboliques dans les dossiers de destination | ✅ | `atomic_write.rs` ; test de lien symbolique à ajouter |
| SEC-D-3 | Presse-papiers : données sensibles | 🟡 | `original_dialog.dart` copie du texte |
| SEC-D-4 | Notifications : contenu affiché écran verrouillé | 🟡 | `mira-host/notifier` |
| SEC-D-5 | Dictée vocale : audio traité localement, pas d'envoi non annoncé | 🟡 | `docs/VOICE.md` |
| SEC-D-6 | Fuzzing des analyseurs (MIME, IMAP, HTML, YAML, protocole démon) | ❌ | aucun dossier `fuzz` |
| SEC-D-7 | Désérialisation des trames du démon : taille maximale, version, fuzz | 🟡 | `mira-protocol/codec.rs`, `frame.rs` |

## Lacunes restantes (❌)

1. **SEC-A06-3** — audit des dépendances Dart pas encore exécuté (workflow en place, OSV-Scanner absent localement).
2. **SEC-A06-6** — pas de SBOM publié.
3. **SEC-A03-7** — pas d'alerte d'hameçonnage sur les liens.
4. **SEC-A09-3** — pas de journal d'audit.
5. **SEC-A05-2** — ACL Windows absentes (`A-FAIRE-WINDOWS.md`).
6. **SEC-A02-7** — pas de chiffrement au repos (décision à prendre).
7. **SEC-D-6** — pas de fuzzing.

Corrigées ou documentées : SEC-A06-2, SEC-A02-8 (documenté), SEC-A04-1, SEC-A05-6.

## Ordre d'instruction proposé

1. **Priorité haute** : SEC-A03-5 (WebView), SEC-A03-4 (assainisseur HTML), SEC-LLM03/04 (agence et exfiltration IA), SEC-A03-3 (CRLF), SEC-A01-4/7 (jeton, portées), SEC-A06-2 (avis RustSec).
2. **Priorité moyenne** : SEC-A10-*, SEC-A02-4/6, SEC-A04-2, SEC-A07-2, SEC-A08-3/5, Android.
3. **Priorité basse / documentation** : modèle de menaces, `SECURITY.md`, SBOM, journal d'audit, fuzzing.

Chaque point instruit reçoit : constat, preuve (fichier:ligne), gravité (CVSS simplifié), correctif, test de non-régression.
