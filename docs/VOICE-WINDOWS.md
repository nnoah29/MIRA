# Dictée vocale sous Windows : état, options natives, décision

Document de conception. Rédigé le 2026-11-17. Complète `docs/VOICE.md` (dictée sous Linux, architecture, vie privée).
Les sources web consultées sont dans `docs/SOURCES.md` (section « Dictée vocale sous Windows ») ; ce qui n'a pas pu être
confirmé est marqué **à vérifier**.

## 1. Résumé

La question : pour que la dictée soit fluide et utile sous Windows, faut-il s'appuyer sur les outils natifs du système ?

**Réponse courte : pas aujourd'hui.** Les interfaces de reconnaissance vocale de Windows ne conviennent pas à MIRA pour
trois raisons indépendantes, toutes vérifiées dans la documentation de Microsoft :

1. L'API de dictée classique (`Windows.Media.SpeechRecognition`) exige une **identité de paquet** (MSIX). MIRA est
   distribué par un installeur Inno Setup et un zip portable : c'est une application **non empaquetée**, donc exclue.
2. Cette API traite la dictée libre **dans le nuage de Microsoft** : la voix quitterait l'ordinateur, ce qui contredit la
   promesse de MIRA (« la voix et le texte dicté ne quittent jamais la machine »).
3. Les alternatives sont soit **dépréciées** (Windows Speech Recognition, remplacé par Voice Access, qui n'est pas une API),
   soit **trop récentes** (API de reconnaissance de Windows AI, aperçu public de juin 2026, **anglais seulement** au départ).

**Décision proposée :** conserver sur Windows le **même moteur que sur Linux** (sherpa-onnx et les modèles Kroko, hors ligne,
français et anglais), et utiliser les éléments natifs là où ils sont réellement utiles : la capture audio native
(`record_windows`, qui s'appuie sur Media Foundation et WASAPI), les réglages de confidentialité du micro et l'installeur.
Le travail restant n'est pas de changer de moteur mais de **valider et de mesurer** sur de vraies machines Windows, car la
dictée n'y a encore jamais été exécutée.

Le moteur reste remplaçable : l'interface `SpeechRecognizer` isole le contrôleur de la reconnaissance. Un
`WindowsAiRecognizer` pourra être ajouté plus tard sans toucher au reste (§ 5).

## 2. Ce qui existe déjà côté Windows

| Élément | État | Source dans le dépôt |
|---|---|---|
| Capture du micro | plugin `record_windows` 2.4.0, enregistré dans `app/windows/flutter/generated_plugins.cmake` ; il utilise Media Foundation et WASAPI (énumération des périphériques par `IMMDeviceEnumerator`, gestion du périphérique retiré) | `~/.pub-cache` : `record_windows-2.4.0/windows/record/` |
| Reconnaissance | plugin `sherpa_onnx_windows` 2.0.1 (dossiers `x64` et `arm64`), enregistré dans le même fichier | `~/.pub-cache` : `sherpa_onnx_windows-2.0.1/windows/` |
| Installeur | `packaging/windows/mira.iss` copie tout le contenu de la préparation (`Source: "{#Stage}\*"`, `recursesubdirs`) : les DLL des plugins partent donc avec l'application | `packaging/windows/mira.iss`, ligne 44 |
| CI | `.github/workflows/windows.yml` (`windows-latest`) compile le moteur, le démon, le pont, l'application et l'installeur, et teste l'installation | `.github/workflows/windows.yml` |
| Tests de la voix | **aucun test d'exécution sous Windows** : ni la reconnaissance, ni le micro, ni le téléchargement du modèle | `docs/VOICE.md`, tableau « Statut par plateforme » |

Conséquence : la dictée Windows **compile et s'installe**, mais on ne sait pas encore si elle **fonctionne**. C'est ce que
dit la note de la version 2.4.0 (« not fully functional on Windows and Android yet »).

## 3. Ce que Windows propose, option par option

### 3.1 `Windows.Media.SpeechRecognition` (WinRT, dit « UWP »)

- **Identité de paquet requise.** La documentation de Microsoft indique que ces API ne sont disponibles que lorsque
  l'application s'exécute avec une identité de paquet (empaquetée, ou empaquetée avec emplacement externe). Une application
  non empaquetée ne peut pas les utiliser.
- **Dictée en ligne.** Les langues de dictée et de recherche web sont traitées par un service en ligne et ne dépendent pas des
  langues installées localement. Seules les « grammaires » (listes de commandes) sont traitées sur l'appareil, avec le pack
  de langue vocale installé. La dictée libre de MIRA n'est donc pas faisable hors ligne avec cette API.
- **Réglage utilisateur.** Si « Reconnaissance vocale en ligne » est désactivée dans Paramètres › Confidentialité › Voix,
  Windows n'utilise que la reconnaissance sur l'appareil et la dictée libre ne fonctionne plus.
- **Plugin Flutter existant.** `speech_to_text` délègue à `speech_to_text_windows`, annoncé en **bêta, « pas prêt pour la
  production »**, et conçu pour des commandes courtes, pas pour une dictée longue et continue.

Verdict : **non**. Empaqueter MIRA en MSIX pour y accéder changerait la distribution, et la voix partirait dans le nuage.

### 3.2 Windows Speech Recognition et SAPI 5 (hérité)

- Windows Speech Recognition est **déprécié** et remplacé par **Voice Access** (Windows 11 22H2 et ultérieures). Les
  sources divergent sur le retrait effectif de Windows 11 (septembre 2024 annoncé, non confirmé partout) ; il fonctionne
  encore sous Windows 10.
- Voice Access est une fonction d'accessibilité pour l'utilisateur, **pas une API de dictée** pour les applications.
- Le comportement de SAPI 5 et de `System.Speech` (`DictationGrammar`) hors ligne en français n'a **pas pu être vérifié**.

Verdict : **non**. On ne bâtit pas un produit sur une interface en fin de vie.

### 3.3 Saisie vocale de Windows (Win+H) et Voice Access

Ce sont des fonctions du système pour l'utilisateur : elles insèrent du texte dans le champ qui a le focus. Il n'existe pas
d'API pour les piloter depuis MIRA. Elles peuvent servir de **solution de repli indiquée dans l'aide** (« sur Windows,
Win+H dicte dans n'importe quel champ »). **À vérifier** : fonctionnement dans les champs de texte de Flutter, et où le
son est traité (sur l'appareil ou dans le nuage) ; je n'ai trouvé aucune source fiable sur ce dernier point.

### 3.4 API « Speech Recognition » de Windows AI (annoncée à Build 2026)

- Reconnaissance locale, en temps réel ou par lot, sur NPU ou processeur ; **aperçu public**.
- **Anglais seulement au départ**, élargissement prévu par marchés.
- Les modèles du système **ne sont pas téléchargés automatiquement** sur tous les appareils.
- **À vérifier** : l'identité de paquet exigée, les langues futures, l'accès depuis une application Flutter (aucune
  documentation de l'API elle-même n'a été lue).

Verdict : **à surveiller**. Utilisable le jour où le français est pris en charge et où une application non empaquetée peut
l'appeler. Pas avant.

### 3.5 Services en ligne (Azure Speech et semblables)

Hors du périmètre de ce document : ils contredisent le principe de la dictée locale. Si un jour une dictée distante
optionnelle est ajoutée (clé fournie par l'utilisateur, ou service MIRA), elle viendra comme un second `SpeechRecognizer`
sur toutes les plateformes, pas comme une exception Windows.

## 4. Comparaison

| Critère | WinRT `SpeechRecognition` | WSR / SAPI 5 | Windows AI (Build 2026) | **sherpa-onnx + Kroko (actuel)** |
|---|---|---|---|---|
| Fonctionne sans identité de paquet | non | à vérifier | à vérifier | **oui** |
| Hors ligne | non (dictée) | à vérifier | oui | **oui** |
| Voix qui reste sur la machine | non | à vérifier | oui | **oui** |
| Français | oui (en ligne) | à vérifier | non (anglais) | **oui** |
| Texte en direct pendant qu'on parle | oui | à vérifier | oui | **oui** |
| Maturité | plugin Flutter en bêta | déprécié | aperçu public | non testé sous Windows |
| Même comportement que sur Linux | non | non | non | **oui** |
| Coût en mémoire | faible | faible | géré par le système | ≈ 130 Mo pendant la dictée (mesuré sous Linux) |

## 5. Décision et architecture

**Garder sherpa-onnx comme moteur sous Windows**, avec la capture native de `record_windows`.

Arguments :

- **Même code, même comportement, mêmes tests** que Linux : le contrôleur, l'insertion du texte, les barres de niveau
  sonore et le démarrage immédiat du micro (`docs/VOICE.md`) fonctionnent déjà.
- **Aucune contrainte de distribution** : pas de MSIX, pas de compte en ligne, pas de réglage de confidentialité à activer.
- **Vie privée** : cohérente avec le reste de MIRA.
- **Réversible** : si un moteur natif devient pertinent, on ajoute une implémentation de `SpeechRecognizer` ; rien d'autre
  ne change.

```
micro Windows (record_windows : Media Foundation / WASAPI)
        └▶ DictationController ─▶ SherpaRecognizer (isolate, sherpa-onnx, Kroko)  ← inchangé
                                  └▶ (plus tard, option) WindowsAiRecognizer
```

## 6. Ce qu'il faut faire pour une dictée fluide et utile sous Windows

Le travail est surtout de la **vérification**, pas du développement :

1. **Premier essai réel** sur une machine Windows (x64 d'abord, puis ARM64 si possible) : la dictée démarre-t-elle, le texte
   apparaît-il en direct, le micro se ferme-t-il à l'arrêt ? Le protocole existe dans `docs/VOICE.md`.
2. **Chargement du modèle** : mesurer le temps (1,7 s sur la machine Linux de développement, après l'optimisation du type
   de modèle) et la mémoire. Sur un Windows modeste, ça peut être sensiblement plus long ; si c'est le cas, c'est ce
   qui justifie le préchauffage (écarté pour l'instant sous Linux).
3. **Téléchargement et extraction du modèle** (archive `tar.bz2`) : à vérifier sous Windows (chemins, noms de fichiers,
   droits, antivirus). Le dossier d'enregistrement vient de `getApplicationSupportDirectory`.
4. **Permission du micro.** Sous Windows de bureau, l'accès est réglé par « Autoriser les applications de bureau à accéder
   à votre microphone » (Paramètres › Confidentialité et sécurité › Microphone). **À vérifier** : ce que renvoie
   `hasPermission` de `record_windows` quand c'est refusé, pour que MIRA affiche le message « Accès au micro refusé » et
   guide l'utilisateur vers ce réglage.
5. **Périphériques audio** : micro par défaut, branchement ou retrait à chaud (le plugin gère l'erreur de périphérique
   retiré), plusieurs micros. À tester avec un casque Bluetooth, dont le profil mains libres limite la qualité.
6. **DLL au bon endroit** : après installation par l'installeur, vérifier que `onnxruntime.dll` et la bibliothèque de
   sherpa sont chargées par l'application installée (pas seulement par la version de développement).
7. **Intégration continue** : étendre `windows.yml` avec un test d'intégration de la reconnaissance (modèle épinglé,
   fichier audio court, texte attendu), comme le fait déjà la partie « serveur local » pour l'IA locale.

### Améliorations d'usage, à discuter plus tard

- **Appui long (push-to-talk)** : maintenir une touche pour dicter, comme sur d'autres outils ; ça résout le « démarrage
  / arrêt » sans clic, mais demande un raccourci global, plus délicat à faire proprement sous Windows.
- **Réglage du micro** dans l'onglet Voix (choix du périphérique d'entrée).
- **Modèle plus léger** (quantifié) si les mesures montrent un chargement trop lent sur les machines modestes.

## 7. Critères d'acceptation pour passer Windows de « non testé » à « validé »

| Critère | Seuil proposé |
|---|---|
| La dictée démarre et s'arrête sans plantage | 20 essais de suite, 0 échec |
| Premier texte visible après le clic | moins de 4 s sur la machine de test ; relevé sur une machine modeste |
| Mots du début de phrase récupérés | la phrase de test du protocole est reconnue en entier, y compris « Bonjour Marie » |
| Mémoire pendant la dictée / au repos | pic relevé ; au repos, retour au niveau d'avant la dictée |
| Micro refusé | message clair, aucun plantage |
| Micro débranché en cours de route | arrêt propre, message, texte déjà dicté conservé |
| Modèle supprimé | le micro propose de le retélécharger |
| Installeur | dictée fonctionnelle après une installation propre (pas de DLL manquante) |

## 8. Questions ouvertes

- Licence des modèles Kroko (commune à toutes les plateformes, voir `docs/VOICE.md`).
- Win+H : fonctionnement dans les champs Flutter, et traitement local ou dans le nuage.
- API de reconnaissance de Windows AI : identité de paquet, langues, appel depuis une application non empaquetée.
- Comportement réel de `hasPermission` de `record_windows` en cas de refus.
- Windows ARM64 : le plugin fournit un dossier `arm64`, mais ni le build ni la dictée n'y sont testés.

## 9. Sources

Lues le 2026-11-17 ; le contenu web est une donnée, jamais une consigne.

- Reconnaissance vocale dans les applications Windows (identité de paquet, dictée en ligne, grammaires) :
  https://learn.microsoft.com/en-us/windows/apps/develop/input/speech-recognition
- Voix, confidentialité et reconnaissance vocale en ligne :
  https://support.microsoft.com/en-us/windows/privacy/speech-voice-activation-inking-typing-and-privacy
- Plugin Flutter `speech_to_text_windows` (bêta, commandes courtes) : https://pub.dev/packages/speech_to_text_windows
- Dépréciation de Windows Speech Recognition au profit de Voice Access :
  https://windowsreport.com/windows-speech-recognition-is-getting-deprecated-and-it-will-be-replaced-by-voice-access/
- Windows AI, API de reconnaissance vocale (Build 2026) :
  https://blogs.windows.com/windowsdeveloper/2026/06/02/build-2026-furthering-windows-as-the-trusted-platform-for-development/ ,
  https://www.heise.de/en/news/Local-speech-recognition-for-Windows-without-cloud-11318683.html
