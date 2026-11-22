# Constats de sécurité MIRA — en clair

Chaque constat dit : **le problème**, **le contexte** (pourquoi ça compte), **les plateformes touchées**, **le correctif**. Ils sont classés du plus grave au moins grave. La checklist technique (`AUDIT-SECURITE-CHECKLIST.md`) en est le complément.

**Ce qui a été vérifié** : lecture du code du moteur, du démon, de l'application Flutter et du manifeste Android. **Ce qui ne l'a pas été** : aucun test en conditions réelles. Les points marqués « à confirmer par un test » sont une hypothèse fondée sur le code, pas une faille démontrée.

**Vue d'ensemble** : la base est sérieuse (TLS partout, secrets dans le trousseau, jeton du démon comparé en temps constant, images distantes bloquées, IA qui *propose* au lieu d'agir, données sensibles masquées avant envoi à l'IA). Les failles ci-dessous sont celles qui restent.

---

## 1. Le corps des mails s'affiche presque sans filtre dans la vue web — gravité HAUTE

**Le problème.** Pour afficher un mail HTML, l'application n'utilise pas la version nettoyée par le moteur. Elle charge le HTML d'origine (décision AG-47, `mira-host/src/original_html.rs`) dans une vue web où JavaScript est activé (`web_mail_body.dart:141`). La seule barrière est une règle de sécurité (CSP) insérée en tête de page, qui interdit tout chargement et tout script.

**Pourquoi c'est un risque.** Un mail est écrit par un inconnu. Si la règle CSP est contournée une seule fois (faille du moteur web, cas particulier non prévu), le script de l'expéditeur s'exécute dans l'application. C'est la faille classique des clients mail : un seul verrou, aucune seconde ligne de défense.

**Trois trous précis déjà visibles dans le code :**

- **a) Les redirections automatiques ne sont pas bloquées.** La règle CSP n'interdit ni `<meta http-equiv="refresh">`, ni l'envoi de formulaires (`form-action`), ni `<base>`. Quand la page tente une navigation, `_onNavigate` (`web_mail_body.dart:~213`) appelle `openLink`, qui ouvre le navigateur du système vers n'importe quelle adresse http(s). Un mail pourrait donc ouvrir un site (suivi, hameçonnage) **sans que l'utilisateur clique**. *À confirmer par un test sur chaque plateforme.*
- **b) Les adresses `data:` sont autorisées sans condition.** Le même `_onNavigate` laisse passer toute navigation vers `about:` ou `data:`. Un lien `data:text/html,…` dans un mail ouvre alors une page de l'expéditeur **sans la règle CSP**, avec JavaScript actif. Elle peut contacter l'extérieur librement (révèle l'adresse IP, confirme la lecture, imite un écran de connexion).
- **c) JavaScript est actif alors que le contenu est hostile.** Il ne sert qu'à mesurer la hauteur de la page et à relayer la molette, et cela donne à une éventuelle exécution de script un canal vers l'application (`MiraMail`).

**Statut (2026-11-18).** Trous **a** et **b** corrigés dans `app/lib/engine/web_document.dart` et `web_navigation.dart` (tests : `web_document_hardening_test.dart`, `web_navigation_test.dart`) : règle CSP complétée, `meta refresh`, `http-equiv` inconnus et `<base>` retirés avant chargement, `about:`/`data:` refusés une fois le document chargé. **Reste le trou c** (nettoyage du HTML d'origine côté moteur, seconde ligne de défense) et la vérification réelle sur chaque plateforme.

**Plateformes.** Linux (WebKitGTK), Android (WebView), Windows (WebView2) ; macOS le sera aussi quand il arrivera. Le rendu natif en widgets (sans vue web) n'est pas concerné.

**Correctif.**
1. Compléter la règle CSP : `form-action 'none'; base-uri 'none'; frame-src 'none'; object-src 'none'`.
2. Dans `_onNavigate`, n'accepter que le chargement initial du document, et refuser tout `data:` ensuite. N'ouvrir le navigateur que pour un lien cliqué par l'utilisateur (refuser les navigations d'une page qui n'est pas en cours d'interaction).
3. Avant chargement, passer le HTML d'origine dans un nettoyage léger côté moteur : retirer `script`, `iframe`, `object`, `embed`, `form`, `base`, `meta refresh`, attributs `on…`, en gardant les styles et la structure. Cela garde le rendu fidèle tout en ajoutant une vraie seconde ligne de défense.
4. Écrire des tests avec des mails piégés (refresh, formulaire, lien `data:`, `<base>`).

---

## 2. Le désabonnement « en un clic » fait confiance à l'expéditeur — gravité MOYENNE

**Le problème.** Si un mail annonce un désabonnement en un clic, MIRA envoie directement une requête vers l'adresse fournie par le mail (`state/unsubscribe.dart`). Le moteur exige du HTTPS (`mime/unsubscribe.rs:30`), mais rien d'autre n'est contrôlé.

**Pourquoi c'est un risque.** Un mail falsifié (usurpation d'un vrai expéditeur) peut fournir n'importe quelle adresse HTTPS, y compris une machine du réseau local (`https://192.168.x.x/…`) ou un site qui, en voyant la requête, sait que votre adresse est active. L'appel part de l'ordinateur de l'utilisateur, donc depuis son réseau privé.

**Plateformes.** Toutes.

**Correctif.**
1. Refuser les adresses privées ou locales (boucle locale, `10.x`, `172.16–31.x`, `192.168.x`, `169.254.x`, noms sans point).
2. Ne proposer le désabonnement automatique que si le mail est authentifié (DKIM valide, comme l'exige la RFC 8058) ; sinon, ouvrir la page dans le navigateur après confirmation.
3. Ne pas suivre les redirections et limiter la taille de la réponse.

---

## 3. Aucune surveillance des failles connues dans les dépendances — gravité MOYENNE

**Le problème.** La CI vérifie les licences et les sources (`ci.sh:42` : `cargo deny check bans licenses sources`), mais **exclut volontairement les avis de sécurité**. Côté Flutter (`pubspec.lock`), aucun contrôle n'existe.

**Pourquoi c'est un risque.** MIRA embarque des composants qui lisent des données hostiles : `ammonia` (HTML), `mail_parser` / `mail_builder` (mails), `zip` / `tar` / `flate2` (archives), `rustls` (TLS), `reqwest`. Une faille publiée dans l'un d'eux resterait inconnue tant que personne ne la cherche.

**Statut (2026-11-22).** Premier passage de `cargo deny check advisories` : **plus de 25 avis réels**, dont `ammonia` (4 avis, dont XSS par SVG/MathML : c'est l'assainisseur du HTML des mails), `lettre` (injection de commandes SMTP, usurpation de réponses STARTTLS, mauvais destinataire), `rustls`/`rustls-webpki` (certificats refusés à tort acceptés, rétrogradation d'échange de clés), `url`, `zstd`, `tokio`, `rusqlite`/SQLite (CVE). **Tous corrigés** par mise à jour du verrou (`ammonia` 4.2.5, `lettre` 0.11.24, `rustls` 0.23.46, `rustls-webpki` 0.103.17, `tokio` 1.54.1, `url` 2.5.9, `zstd` 0.14.4, `webpki-roots` 1.0.12) et montée de `rusqlite` 0.40 → 0.41 (SQLite embarqué corrigé), sans changement de code ; tests ciblés HTML, SMTP et stockage verts. **Restent 4 avis « non maintenu »** (`chrono`, `zbus_names`, `zvariant`, `zvariant_derive`), sans correctif possible, inscrits avec justification dans `engine/deny.toml`. Ajoutés : `.github/workflows/securite.yml` (hebdomadaire + avant release : avis RustSec et OSV sur `pubspec.lock`) et `.github/dependabot.yml`. **Reste** : SBOM par release, et l'audit Dart n'a pas encore tourné (OSV-Scanner absent de cette machine).

**Plateformes.** Toutes (c'est la chaîne de fabrication).

**Correctif.**
1. Ajouter `cargo audit` (ou `cargo deny check advisories`) dans une tâche planifiée hebdomadaire et avant chaque release, puisque l'exclusion de la CI courante est motivée par l'absence de réseau.
2. Ajouter un scan des dépendances Dart (par exemple OSV-Scanner sur `pubspec.lock`).
3. Activer Dependabot ou Renovate, et publier un SBOM avec chaque release.

---

## 4. Données de mail non chiffrées sur le disque — gravité MOYENNE

**Le problème.** La base locale est du SQLite ordinaire (`rusqlite` avec `bundled`, aucun chiffrement). Les mails, les brouillons, les contacts et la mémoire de l'IA y sont lisibles par quiconque accède au fichier.

**Pourquoi c'est un risque.** Un ordinateur volé ou perdu sans chiffrement de disque, une sauvegarde copiée, un autre programme qui lit le dossier de l'utilisateur : tout le courrier est exposé. Les secrets (mots de passe, jetons) ne sont pas dans la base : ils sont dans le trousseau, ce qui est bien.

**Plateformes.** Linux, Windows, macOS. **Android** est protégé par l'isolation des applications (et `allowBackup="false"`).

**Correctif.** Deux options, à trancher :
- chiffrer la base avec SQLCipher, clé conservée dans le trousseau (coût : build et performances) ;
- ne pas chiffrer, mais **le documenter clairement** et recommander le chiffrement du disque. C'est ce que font la plupart des clients mail.

---

## 5. Windows : protection du jeton du démon à confirmer — gravité MOYENNE (à confirmer)

**Le problème.** Sous Linux, le fichier du jeton et le socket sont réservés à l'utilisateur (permissions 0600/0700). Sous Windows, la vérification est volontairement vide (`token/file.rs:44`, « phase Windows ») et le démon écoute en TCP sur `127.0.0.1`.

**Pourquoi c'est un risque.** Le jeton est l'unique clé d'accès au démon, donc à tous les mails. Sur un poste partagé (plusieurs sessions, bureau à distance), un autre utilisateur peut atteindre le port local ; seul le jeton l'arrête. Si les droits du dossier qui contient le jeton sont trop larges, il le lit.

**Plateformes.** Windows. **macOS** devra être traité dès le départ (socket Unix dans le dossier de l'utilisateur, pas de TCP).

**Correctif.** Fixer explicitement les droits du dossier et du fichier du jeton à l'utilisateur seul (ACL), ou remplacer le TCP par un tube nommé (*named pipe*) avec ACL. Tester avec un second compte Windows.

---

## 6. Le démon accepte un nombre illimité de connexions — gravité BASSE

**Le problème.** La boucle d'acceptation (`server/accept.rs`) crée une tâche par connexion sans plafond. Il y a bien un délai de 5 s pour le premier message et une taille maximale de 8 Ko, mais pas de limite du nombre de connexions.

**Pourquoi c'est un risque.** Un programme local malveillant peut ouvrir des milliers de connexions et épuiser la mémoire du démon (déni de service). Il ne peut rien lire sans le jeton, qui fait 256 bits et se compare en temps constant.

**Plateformes.** Toutes, surtout là où le TCP est utilisé.

**Correctif.** Plafonner les connexions simultanées (par exemple 16) et couper celles qui échouent à l'authentification.

---

## 7. Les identifiants OAuth embarqués ne sont que masqués — gravité BASSE

**Le problème.** Le secret client OAuth est chiffré dans le binaire, mais la clé de déchiffrement est livrée juste à côté (`mira-assembly/src/oauth_embedded.rs`). N'importe qui peut donc le retrouver.

**Pourquoi c'est un risque limité.** Pour une application installée sur l'ordinateur de chacun, ce n'est pas un vrai secret : Google et Microsoft le savent. La protection réelle vient de PKCE, qui est en place. Le danger serait de s'imaginer que ce secret protège quelque chose.

**Plateformes.** Toutes.

**Correctif.** Ne rien changer au mécanisme, mais le documenter comme simple masquage, ne jamais y mettre de secret réellement confidentiel, et préférer un client « public » sans secret quand le fournisseur le permet.

---

## 8. Pas d'alerte d'hameçonnage sur les liens — gravité BASSE (fonction manquante)

**Le problème.** Quand le texte d'un lien affiche `banque.fr` et que la cible est `banque-secure.xyz`, rien ne prévient l'utilisateur. Idem pour les adresses à caractères trompeurs (homographes).

**Plateformes.** Toutes.

**Correctif.** Avant d'ouvrir un lien, comparer texte visible et domaine cible ; afficher la vraie adresse et demander confirmation en cas d'écart. Ajouter un avertissement sur les expéditeurs dont l'authentification échoue.

---

## 9. Manques de documentation et de processus — gravité INFORMATIVE

- Pas de **`SECURITY.md`** : personne ne sait comment signaler une faille.
- Pas de **modèle de menaces** écrit (qui attaque quoi, par quel chemin).
- Pas de **fuzzing** : on n'envoie pas de données déformées aux lecteurs de MIME, d'IMAP, de HTML, de YAML (règles) ni au protocole du démon.
- Pas de **journal d'audit** (échecs de connexion au démon, ajout de compte, envois).
- **Android** : le manifeste est correct (`allowBackup="false"`, pas de trafic non chiffré, secrets dans le Keystore). Il reste à vérifier les réglages de la WebView et les droits de l'activité exportée.

**Correctif.** Les écrire ; le `SECURITY.md` et le modèle de menaces prennent une demi-journée.

---

## Points déjà bons (pour mémoire)

- Jeton du démon comparé en temps constant (HMAC) ; **correction de ma première passe** qui l'avait signalé à vérifier.
- Le moteur refuse toute connexion en clair, sans contournement de certificat.
- Android : secrets chiffrés par une clé du Keystore.
- L'IA ne fait que **proposer** (envoi, suppression) ; les mails, IBAN, cartes et codes à usage unique sont masqués avant envoi (`ai/masking`).
- Archives extraites avec contrôle des chemins et des liens, limite de taille.
- Téléchargement des modèles IA vérifié par somme de contrôle.
- URL des connecteurs IA nettoyées : pas d'identifiants dans l'adresse, pas de clair hors machine locale.

## À tester avant de conclure (non démontré)

| Sujet | Hypothèse | Test |
|---|---|---|
| Injection d'en-têtes (retours à la ligne) dans un sujet ou une adresse | `mail_builder` encode correctement | envoyer `Sujet\r\nBcc: x` et lire le message produit |
| Injection de commandes IMAP | `quote_mailbox` protège toutes les commandes | noms de boîtes avec `"`, `\r\n`, `{`, tester aussi la recherche |
| Règles YAML importées | profondeur et taille limitées | fichier YAML géant ou imbriqué |
| Mail MIME très imbriqué ou énorme | limites présentes | mail avec 10 000 parties |
| Injection de prompt par un mail | masquage + propositions suffisent | mail hostile ordonnant d'envoyer un résumé à l'extérieur |
| Réglages de la WebView Android | valeurs par défaut sûres | vérifier accès aux fichiers et contenu mixte |

## Ordre de correction conseillé

1. **Constat 1** (vue web) : c'est le seul qui touche directement un attaquant anonyme par un simple mail.
2. **Constat 3** (surveillance des dépendances) : rapide, grand gain.
3. **Constat 2** (désabonnement).
4. **Constat 5** (Windows) avant de diffuser sur Windows.
5. Constats 6 à 9, et la décision sur le chiffrement de la base (4).
