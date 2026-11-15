# Dictée vocale — conception

Statut : **à approuver**. Branche `feat/voice-dictee-conversation` (issue de `dev`), worktree `MIRA-voice`.
Plan d'implémentation associé : `docs/superpowers/plans/2026-11-15-dictee-vocale.md`.

## Intention

Dicter au lieu de taper, dans deux endroits : le **corps d'un brouillon** (composeur) et le **champ de
saisie du chat avec l'IA**. L'utilisateur parle, les mots apparaissent à l'écran **pendant qu'il parle**.
Motivation donnée par l'utilisateur : le **confort de la dictée**. L'IA répond toujours par écrit ;
il n'y a pas de synthèse vocale.

## Buts

- Dictée en direct, **locale** : la voix ne quitte jamais la machine.
- **Français et anglais à poids égal** : même qualité attendue, mêmes écrans, même taille de modèle.
- Un seul code, **portable** : Linux (validé), Windows, Android ; macOS dès que la plateforme existe dans `app/`.
- Texte directement utilisable : majuscules et ponctuation viennent du modèle.

## Non-buts

Réponse vocale de l'IA, mode mains libres, mot d'éveil, commandes vocales (« point à la ligne »),
reconnaissance distante, détection automatique de la langue, version web, mots favorisés (contacts)
dans cette première livraison.

## Anti-buts (échec même si « ça marche »)

- Une donnée vocale ou un texte dicté qui part sur le réseau.
- Un modèle téléchargé sans que l'utilisateur l'ait demandé et confirmé.
- Une dictée qui écrit dans un champ fermé, ou qui écrase ce que l'utilisateur tape pendant ce temps.
- Une interface qui fige pendant la reconnaissance.

## Contraintes

- La dictée vit **côté Flutter** (`app/lib/voice/`), pas dans le moteur Rust : aucune logique mail, c'est
  un mode de saisie. Le moteur et le pont ne changent pas.
- Licence des modèles Kroko « communautaires » : **CC-BY-SA**. Ils ne sont jamais commités ni embarqués ;
  ils se téléchargent à la demande, avec attribution visible dans « À propos ».
  À confirmer pendant l'implémentation (tâche 10) : que les deux archives pinnées relèvent bien de cette licence.
- Réseau : le seul accès est le téléchargement du modèle, après confirmation explicite.
- Tests uniquement contre des serveurs locaux `127.0.0.1` ; aucun test ne dépend du micro ni des modèles
  (un test d'intégration optionnel les utilise si `MIRA_VOICE_MODELS` est défini).
- Conventions de l'app : textes par `t()`, `//!` en tête de fichier, une ligne `///` en français par fonction.

## Approches envisagées

| | Approche | Verdict |
|---|---|---|
| A | Reconnaissance dans le moteur Rust (Whisper en serveur local, audio via le pont) | Écartée : Whisper n'est pas un modèle en flux (pas de mots en direct), un processus externe est fragile sur Android, et c'est de la tuyauterie pour un outil sans logique mail. |
| B | Service distant via un fournisseur IA déjà configuré | Écartée : la voix quitterait la machine ; contraire à la promesse de confidentialité. Pourra revenir comme option via le moteur (les clés y vivent). |
| C | API vocales natives de chaque système | Écartée : comportement différent par plateforme, certaines envoient l'audio chez un tiers, rien de fiable sur Linux. |
| **D** | **Reconnaissance locale en flux dans l'app Flutter (`sherpa_onnx`, modèles Kroko), capture par `record`** | **Retenue** (choix de l'utilisateur, validé par un spike). |

## Résultats du spike (Linux, voix de l'utilisateur, 15 novembre 2026)

- Kroko FR et EN (57 Mo chacun) : direct fluide, mises à jour par paquets d'environ 1,2 s,
  décodage 40–350 ms par paquet, ponctuation et casse produites par le modèle.
- Les modèles de référence plus gros (FR 398 Mo, EN 128 Mo) sont **moins précis**, sans ponctuation, tout en
  majuscules : écartés.
- Points faibles mesurés : noms propres rares (« Sylvalaine »), sigles épelés (« DG »), termes anglais au
  milieu d'une phrase française (« GitHub » entendu « auditeur »), accent non natif, « November ».
- Mots favorisés : sans le fichier SentencePiece de Kroko, gain non démontré → reporté.

## Décisions prises avec l'utilisateur

- Dictée seulement (pas de voix en sortie). Locale. Français et anglais à égalité.
- Côté Flutter plutôt que dans le moteur. Portable : Linux, Windows, macOS, Android.

## Mes choix (à corriger si besoin)

- **Bouton micro** : on appuie pour démarrer, on rappuie pour arrêter. Pas d'appui maintenu.
- **Langue** : celle de l'interface par défaut, sélecteur FR/EN dans les réglages, mémorisé. Pas de détection.
- **Modèles** : téléchargés par langue, à la demande, dans le dossier de support de l'app. Empreinte SHA-256 vérifiée.
- **Insertion** : le texte partiel s'affiche à la position du curseur et se remplace à chaque mise à jour ;
  à la fin d'une phrase il est validé et suivi d'un espace. Si l'utilisateur tape pendant ce temps, le
  partiel est validé tel quel et la dictée repart du nouveau curseur.
- **Une seule dictée à la fois** dans toute l'app. Elle s'arrête à l'envoi, à la fermeture du champ, à un
  changement de langue, et au bout de 5 minutes.
- **Reconnaissance dans un isolate** pour ne jamais figer l'interface.
- Apostrophes normalisées en `'` ; aucune majuscule ni ponctuation ajoutée par nous.
- Pas de nouvelle dépendance réseau : téléchargement par `dart:io`. Nouvelles dépendances :
  `sherpa_onnx`, `record`, `archive` (tar.bz2), `crypto` (SHA-256).

## Modèles pinnés

Source : `https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/<archive>`

| Langue | Archive | Octets | SHA-256 |
|---|---|---|---|
| fr | `sherpa-onnx-streaming-zipformer-fr-kroko-2025-08-06.tar.bz2` | 57 220 361 | `e6ffd3dc43725cd6c8137b05c739f15607d0df946b9b90eb141e10059efca024` |
| en | `sherpa-onnx-streaming-zipformer-en-kroko-2025-08-06.tar.bz2` | 57 267 600 | `c8676e5ff9ac2a85296e53ee0fd4d5fb1db6770e7a7647166eeafe349ade6834` |

Contenu utile : `encoder.onnx`, `decoder.onnx`, `joiner.onnx`, `tokens.txt` (≈ 68 Mo décompressés).

## Laissé au réalisateur

Mise en page exacte du bouton micro dans la barre du composeur et du chat ; animation d'écoute ;
texte des messages d'erreur ; découpage interne des fichiers tant que les interfaces du plan sont respectées.

## Risques connus

1. `record` sous Linux peut exiger des outils système (PulseAudio/PipeWire) : repli prévu, un lecteur
   direct de `parecord`, déjà validé par le spike.
2. Poids et fluidité sur un Android modeste : à mesurer sur appareil (tâche 11).
3. Taille des paquets : bibliothèque native `onnxruntime` ajoutée par plateforme.
4. Qualité sur noms propres : limite du modèle de 57 Mo, documentée à l'utilisateur dans les réglages.
