# Système de design MIRA — spécification de reproduction

Ce document décrit le design de MIRA de façon suffisamment précise pour le
reproduire dans une autre application, quel que soit son framework. Il sépare
ce qui constitue la grammaire visuelle (à conserver) de l'identité de marque
(à remplacer si nécessaire).

La référence implémentée est l'application Flutter dans `app/`. Les captures
dans `docs/screenshots/` montrent les états de référence : boîte vide,
message ouvert et paramètres.

## 1. Intention et règles fondatrices

MIRA est un **brutalisme chaleureux de bureau** : dense mais lisible,
fonctionnel mais expressif. L'interface doit évoquer un atelier de travail,
pas une application générique et molle.

Ses invariants visuels sont les suivants :

- Des surfaces presque noires ou ivoire, jamais noir/blanc purs en fond.
- Des blocs rectangulaires, à angles droits et bordures fines ; l'arrondi est
  exceptionnel et réservé au contenu externe (image HTML, document), jamais
  à la navigation, aux boutons ou aux cartes applicatives.
- Une unique couleur d'accent, chaude par défaut, qui signifie l'action
  primaire, la sélection et l'état actif. Elle ne sert jamais de décoration
  gratuite.
- Une hiérarchie typographique très marquée : titre condensé, texte humain,
  métadonnée technique.
- Une densité assumée : peu de grands aplats, beaucoup de structure par les
  bordures, l'alignement et des espacements réguliers.
- Pas de dégradé d'interface, pas d'effet verre, pas de grosses ombres, pas de
  pastilles très arrondies, pas de ripple Material visible.

Pour dériver une autre app, garder cette grammaire et changer uniquement le
nom, le logo, les textes métier, les icônes éventuellement, et l'accent de
marque. Ne mélangez pas cette grammaire avec une bibliothèque de composants
aux coins arrondis par défaut.

## 2. Fondations visuelles

### Palette

Ne manipuler que des couleurs sémantiques. Les valeurs de référence sont :

| Rôle | Sombre | Clair | Usage |
| --- | --- | --- | --- |
| `bg` | `#1B1B1B` | `#F3F1EA` | fond principal et pages |
| `bgSide` | `#1E1E1E` | `#EAE7DE` | navigation latérale |
| `surface` | `#242424` | `#FBFAF6` | carte, survol discret |
| `surface2` | `#2A2A2A` | `#E3E0D6` | sélection passive, popover |
| `line` | `#333333` | `#CDC9BD` | bordure visible 1 px |
| `lineSoft` | `#2B2B2B` | `#DCD8CC` | séparateur structurel 1 px |
| `text` | `#ECEBE6` | `#1D1C1A` | texte à forte importance |
| `body` | `#D6D5CD` | `#34322E` | texte long, contenu de mail |
| `muted` | `#ADACA5` | `#4B4943` | contrôle inactif, texte secondaire |
| `dim` | `#8B8A84` | `#6B685F` | métadonnée, aide, compteur discret |
| `accent` | `#E8542B` | `#D6431A` | action, sélection, focus, lien |
| `onAccent` | `#1B1B1B` | `#FFFFFF` | texte/icône sur l'accent |
| `toastBg` | `#ECEBE6` | `#1D1C1A` | fond des notifications |
| `toastFg` | `#1B1B1B` | `#F3F1EA` | texte des notifications |
| `toastAccent` | `#BB3510` | `#FF8A63` | action dans une notification |

La teinte d'accent translucide, `accentSoft`, vaut l'accent à 12 % en sombre
et à 10 % en clair. C'est le fond de l'état sélectionné ; sa bordure et son
texte restent en accent opaque. Ne pas réduire simultanément l'opacité du
texte : l'état actif doit rester net.

Si l'autre application propose une couleur d'accent choisie par l'utilisateur,
ajuster légèrement sa luminance en thème clair pour la rendre plus dense, puis
choisir `onAccent` par contraste. Le minimum de contraste utile avec un fond
est de 3,2:1 ; pour du texte courant, viser 4,5:1.

### Typographie

Les trois familles font partie du langage, pas d'un choix décoratif :

| Rôle | Police | Graisses | Emploi |
| --- | --- | --- | --- |
| Affichage | Anton Regular | 400 | marque, titres de pages, nombres majeurs |
| Interface | Hanken Grotesk | 400, 500, 600, 700 | libellés, champs, lecture, corps |
| Métadonnée | JetBrains Mono | 400, 500, 700 | boutons, onglets, raccourcis, dates, compteurs |

Les polices sont embarquées dans `app/assets/google_fonts/`. Pour une autre
plateforme, fournir les mêmes fichiers sous licence compatible ou fixer des
substituts métriquement proches ; ne pas laisser le système substituer une
sans-serif aléatoire.

| Niveau | Police et valeur | Règle |
| --- | --- | --- |
| Nombre majeur (sidebar) | Anton 52 / 1 | un seul chiffre ou compteur important |
| Marque | Anton 30 / 1, suivi 0,06 em | seul `RA` prend l'accent |
| Titre de boîte | Anton 34 / 1 | capitales, suivi discret |
| Titre de page | Anton 24–30 / 1 | capitales ; 24 dans un panneau, 30 à l'accueil |
| Texte interface | Hanken 15 / 1,5 | valeur par défaut |
| Texte secondaire | Hanken 13,5–14 / 1,5 | explication, adresse, aperçu |
| Texte riche | Hanken 16 / 1,8 | composition et lecture longue |
| Métadonnée | JetBrains Mono 11 / 1,5 | capitales, suivi 0,16 em |
| Petit libellé | JetBrains Mono 10–10,5 | titre de groupe, date, badge |

Les libellés mono sont en majuscules et très espacés. Les valeurs, noms et
phrases restent en Hanken : ne mettez jamais un paragraphe ou une information
critique uniquement en petites capitales mono. Centrer verticalement le texte
sur une hauteur de ligne entière ; cela évite le rendu flottant caractéristique
des polices condensées.

### Géométrie, bordures et relief

La grille d'espacement est fondée sur 2 px, avec les pas usuels
`4, 6, 8, 10, 12, 14, 16, 20, 24, 32, 40`.

- Tous les séparateurs et contours de composants font **1 px**.
- Les composants applicatifs ont un rayon de **0 px**.
- Les petits carrés de couleur, cases et avatars sont carrés. L'avatar est un
  carré coloré avec initiales, jamais un portrait rond.
- Une bordure gauche de 2 px en accent indique l'élément de navigation actif.
- L'ombre est un signal d'élévation, jamais une décoration : popover
  `0 14px 36px rgba(0,0,0,.40)`, panneau poussé `-6px 0 24px rgba(0,0,0,.35)`.
- Une bordure en tirets emploie un trait de 3 px suivi de 3 px ; elle sert aux
  pièces jointes et zones de dépôt.

## 3. Mise en page responsive

La structure est une application de travail à trois zones, non une série de
cartes flottantes.

```text
┌─────────────── 64 px : recherche, compte, actions globales ───────────────┐
│ sidebar 248 px │ liste 430 px                  │ espace de travail         │
│ navigation     │ titre + filtres + actions      │ lecteur / composer / ...  │
│ persistante    │ liste défilante                │                           │
└────────────────┴───────────────────────────────┴───────────────────────────┘
```

| Largeur | Navigation | Contenu |
| --- | --- | --- |
| `> 1350 px` | sidebar fixe 248 px ; libellé du compte visible | liste 430 px + panneau droit permanent |
| `1101–1350 px` | sidebar fixe 248 px ; compte réduit à l'avatar | liste 430 px + panneau droit permanent |
| `761–1100 px` | sidebar fixe 248 px | une liste de base ; un détail recouvre la liste en glissant depuis la droite |
| `≤ 760 px` | sidebar dans un tiroir gauche | une seule vue à la fois ; panneau de détail poussé depuis la droite |

Règles de dimensions :

- La barre supérieure fait **64 px** de haut, avec 32 px d'inset horizontal sur
  bureau et 12 px sur mobile.
- La sidebar fait **248 px**. Son logo commence à `24 px` du bord et `22 px`
  du haut. Sur mobile, le tiroir fait `86 %` de l'écran, plafonné à **300 px**.
- La liste fait **430 px** seulement en disposition à trois panneaux. Son
  contenu a 24 px de gouttière sur bureau et 16 px en largeur réduite.
- Le panneau de travail prend le reste et emploie 32 px de gouttière sur grand
  écran, 16 px quand la disposition est empilée.
- Une page de premier lancement est centrée et limitée à **520 px** ; son
  padding extérieur est 24 px.

La zone de travail reste opaque : lorsqu'un détail apparaît dans la mise en
page étroite, la liste sous-jacente se décale à gauche de 22 % de sa largeur et
reçoit un voile noir au plus à 30 %. Elle n'est pas redimensionnée.

## 4. Composants fondamentaux

Construire ces primitives une fois avant les pages. Elles sont le contrat qui
maintient la cohérence de la seconde application.

### Icônes et logo

Les icônes sont des SVG en trait : viewBox 24 × 24, trait rond de **1,6 px**,
extrémités et jointures rondes, taille normale 16 px. Elles héritent de la
couleur du contexte : `muted` au repos, `text` au survol, `accent` lorsqu'elles
représentent un état actif. Une étoile active peut être remplie.

Le logo MIRA est un cadre ouvert, anguleux, dessiné au trait, avec un `M` Anton
centré. Son trait contient un dégradé uniquement dans le logo, de l'accent
éclairci à l'accent assombri. Tailles de référence : 48 px dans la sidebar,
56 px à l'accueil. Dans une autre marque, remplacer le tracé et le glyphe mais
conserver l'encombrement carré et cette retenue.

### Bouton principal et bouton secondaire

| Variante | Fond | Bordure | Texte | Padding |
| --- | --- | --- | --- | --- |
| Primaire (`go`) | `accent` | `accent` | `onAccent`, mono gras | 16 × 10 px |
| Secondaire | transparent | `line` | `muted`, mono | 16 × 10 px |
| Secondaire survolé | `surface` | `dim` | `text` | identique |
| Secondaire actif | `accentSoft` | `accent` | `accent` | identique |
| Icône seule | même logique | même logique | icône 16 px | carré 38 px, ou 30 px pour une action de liste |

L'icône précède le texte avec 10 px d'écart. Le bouton primaire de sidebar
utilise 13 px de padding sur tous côtés, prend toute la largeur disponible et
reste rectangulaire. Le clic donne une réduction à 94 % pendant 90 ms. Aucun
état désactivé ne doit ressembler à une action active : enlever l'action et
utiliser `dim`/opacité, avec une explication si le motif n'est pas évident.

### Navigation, onglet, filtre et sélection

- **Entrée de navigation** : 12 px horizontal, 8,25 px vertical ; icône puis
  14 px puis libellé Hanken 15 ; compteur mono à droite. Sélection =
  `surface2` + liseré gauche 2 px accent + icône/compteur accent.
- **Titre de section** : mono 10,5, suivi 0,18 em, padding `12 12 10`.
- **Pill/filtre** : rectangle bordé, mono 11 ; padding `11 × 6,25`. Inactif
  `line`, actif `accentSoft` + bord et texte accent. Espacement de 6 px.
- **Onglets segmentés** : segments rectangulaires accolés, séparés par 1 px de
  `line`. L'actif reçoit `accentSoft` et le texte accent ; aucun indicateur
  arrondi ou souligné épais.
- **Checkbox** : carré 14 px, bord `dim` au repos ; fond accent si coché ;
  état indéterminé = barre accent de 8 × 2 px.

### Champ, carte, popover et notification

- **Champ** : pas de contour interne si le contexte fournit déjà une ligne ;
  curseur 1 px accent ; Hanken 15 ; placeholder `muted`. Une ligne de groupe
  possède des paddings verticaux de 16 px et des séparateurs `lineSoft`.
- **Carte applicative** : fond `surface`, bord `line`, rayon zéro. Exemple :
  message d'un fil, réglage groupé, document dans le lecteur.
- **Popover** : `surface2`, bord `line`, padding externe 6 px, largeur minimale
  220 px, largeur maximale 320 px. Il s'ancre 6 px sous le contrôle, se replie
  au-dessus s'il manque de place et garde 8 px de marge écran.
- **Item de popover** : padding `12 × 9`, texte Hanken 14, coche trait 14 px
  en accent ; au survol fond `surface`.
- **Toast** : contraste inversé (`toastBg`/`toastFg`), texte court, action en
  `toastAccent`. Il ne masque pas une action indispensable.

## 5. Composition des écrans

### Sidebar persistante

Elle est composée dans cet ordre : marque, grand compteur non lu, statut compact
« à répondre », bouton primaire pleine largeur, navigation de boîte, puis
groupes de libellés. Les libellés utilisent un carré coloré de 8 px ; les
libellés locaux peuvent employer le diamant de 8 px.

La sidebar est une zone de pilotage, pas un tableau de bord : ne lui ajouter ni
cartes marketing ni graphiques. Conserver les compteurs alignés à droite et les
icônes cohérentes avec les actions du contenu.

### Barre supérieure

À gauche, recherche pleine largeur disponible : icône de recherche, écart
14 px, champ sans encadrement, bouton filtres puis effacement conditionnel.
À droite : sélecteur de compte, bouton assistant, bouton paramètres. Les deux
derniers sont des boutons secondaires ; sur mobile, ils deviennent icône seule
avec tooltip. La bordure basse `lineSoft` sépare la barre sans la transformer
en carte.

### Liste et espace vide

La liste a toujours cette séquence : titre Anton, sous-titre mono (compteur),
filtres, barre d'actions de 1 px encadrée, puis items défilants. Les regroupements
de date sont mono, 10–11 px. Chaque ligne de message est séparée par `lineSoft`.
Un message sélectionné emploie l'accent comme fin liseré latéral et une surface
un peu plus claire, jamais un gros aplat orange.

L'espace vide du panneau droit se centre verticalement : icône 44 px `dim`,
instruction mono, courte phrase d'aide avec touche clavier, puis bouton
primaire. Il doit paraître calme et intentionnel, jamais comme une erreur.

### Lecteur, composition, assistant et réglages

- **Lecteur** : barre d'actions en haut séparée par une ligne ; le premier
  geste (répondre) est primaire. Les autres sont secondaires ou icône seule.
  Un message est une carte `surface` bordée ; en-tête avec avatar carré 40 px,
  identité, date mono et chevron. Les pièces jointes sont dans des boîtes à
  tirets, pas des chips arrondies.
- **Composition** : titre Anton 24, état de sauvegarde mono, fermeture icône.
  Destinataires organisés en lignes structurées, objet séparé par deux lignes,
  corps Hanken 16 / 1,8. La barre de formatage est une petite palette `surface2`
  avec bordure et ombre, pas une barre d'outils système.
- **Assistant** : conversation en colonne, bulles rectangulaires. Bulle de
  l'utilisateur = `accentSoft`; réponse = `surface`. Les sources, appels outils
  et consentements restent des éléments mono/encadrés qui n'écrasent pas le
  texte de réponse.
- **Réglages** : titre Anton, onglets mono en ligne, grande carte bordée pour
  les paramètres associés. Une ligne de réglage comprend titre Hanken 15,
  explication 13,5 `dim`, et contrôle aligné à droite. Sur mobile, empiler le
  contrôle après le texte avec un espacement explicite.
- **Accueil** : une colonne étroite, logo seul, titre Anton, trois promesses
  sous forme texte (pas de cartes), sélecteur de langue, action primaire puis
  action secondaire.

## 6. États, mouvement et retour d'action

L'état visuel est explicite et sobre :

| Situation | Traitement |
| --- | --- |
| Survol | fond `surface`, bordure `dim`, texte `text` |
| Sélection persistante | `accentSoft`, bordure/texte accent |
| Action primaire | aplat accent, texte `onAccent` |
| Erreur/attention | accent + message précis ; ne pas introduire un rouge secondaire sans raison |
| Chargement | squelette discret `surface2`, angles jusqu'à 3 px seulement |
| Aucun résultat | icône dim + phrase d'aide + action de sortie |
| Succès annulable | toast inversé avec action visible |

Les animations ne doivent jamais retarder une action :

| Élément | Mouvement | Durée |
| --- | --- | --- |
| Survol/état de bouton, navigation, pill | interpolation de couleur | 130 ms |
| Pression bouton | échelle 1 → 0,94 → 1 | 90 ms |
| Changement de panneau large | fondu + glissement horizontal de 1,2 % | 200 ms |
| Popover | fondu + montée de 4 px | 120 ms |
| Ouverture d'un détail étroit | pousse depuis la droite | 340 ms |
| Fermeture d'un détail étroit | retour vers la droite | 280 ms |
| Tiroir mobile | suit le doigt, puis glisse | 280 ms |
| Carte de fil repliable | changement de hauteur | 220 ms |

Respecter `prefers-reduced-motion` / le réglage système : dans ce cas, rendre
ces transitions instantanées. Un tooltip apparaît après 700 ms ; il est requis
pour tout bouton icône seule sur ordinateur.

## 7. Interaction et accessibilité

Le design est pensé pour souris, clavier et tactile :

- Cibles tactiles : 38 px minimum pour un contrôle compact ; 44 px à privilégier
  sur mobile. Les actions de liste peuvent descendre à 30 px seulement si elles
  sont dans une barre dense et restent utilisables à la souris.
- Les contrôles doivent avoir un nom accessible ; une icône sans libellé visuel
  a un tooltip et un libellé programmatique.
- Ne transmettre aucun état par la couleur seule : ajouter coche, liseré,
  icône, compteur ou texte.
- Garder le focus clavier très visible en réutilisant l'accent ; ne pas
  supprimer l'indicateur de focus du navigateur/framework sans le remplacer.
- Contraste : 4,5:1 pour le texte courant, 3:1 minimum pour les icônes et
  bordures porteuses de sens.
- Éviter les actions destructrices cachées au survol et toujours confirmer les
  actions irréversibles.

Raccourcis de la grammaire MIRA : `/` recherche, `c` créer, `j`/`k` déplacer
la sélection, `Entrée` ouvrir, `r` répondre, `a` répondre à tous, `f`
transférer, `e` archiver, `#` supprimer, `s` favori, `u` lu/non lu, `x`
sélection multiple, `Échap` ferme d'abord l'élément le plus haut. Les champs
éditables gardent naturellement leurs propres raccourcis.

## 8. Contrat d'implémentation

Pour empêcher les écarts dans une autre app, créer ces couches, dans cet ordre :

1. **Tokens** : couleurs, fontes, tailles, espacements, bordures, durées et
   breakpoints dans un seul module. Aucun hexadécimal dans un écran.
2. **Primitives** : `Button`, `IconButton`, `NavItem`, `Pill`, `Segmented`,
   `FieldRow`, `Card`, `Popover`, `Toast`, `Avatar`, `EmptyState`.
3. **Coque** : `Sidebar`, `Topbar`, `ListPanel`, `WorkPanel`, tiroir mobile et
   pile de détail étroite.
4. **Patrons de page** : lecture, formulaire/éditeur, conversation, réglages,
   accueil. Une nouvelle fonctionnalité choisit un patron ; elle n'invente pas
   son propre chrome.
5. **Tests visuels** : captures aux largeurs de rupture, pour les états repos,
   survol, actif, vide, chargement, erreur et thème clair/sombre.

Dans MIRA, les références de code sont :

| Sujet | Référence |
| --- | --- |
| Tokens, thèmes et typo | `app/lib/theme/app_theme.dart` |
| Icônes | `app/lib/theme/icons.dart` |
| Logo | `app/lib/ui/mira_logo.dart` |
| Primitives | `app/lib/ui/widgets.dart` |
| Coque et breakpoints | `app/lib/ui/shell.dart`, `sidebar.dart`, `side_drawer.dart`, `push_stack.dart` |
| Liste | `app/lib/ui/mail_list.dart` |
| Lecteur | `app/lib/ui/reader.dart` |
| Composition | `app/lib/ui/composer.dart` |
| Popovers, modales et toasts | `app/lib/ui/overlays.dart` |
| Accueil | `app/lib/ui/welcome_page.dart` |

## 9. Recette de fidélité avant livraison

Comparer des captures fixes avec la référence aux formats suivants :

| Vue | Largeur à contrôler | États minimums |
| --- | --- | --- |
| Bureau | 1920 px | boîte vide, message ouvert, paramètres, popover |
| Bureau compact | 1101 px | panneau droit sans débordement |
| Tablette | 1100 px | ouverture/fermeture du détail poussé |
| Mobile | 760 px | tiroir, barre compacte, détail, formulaire |
| Mobile étroit | 390 px | aucun texte tronqué sans ellipse volontaire |
| Thèmes | chaque format | sombre et clair, accent alternatif |

La reproduction est conforme seulement si les critères suivants sont tous vrais :

- [ ] aucun composant applicatif n'est arrondi ; les coins et bordures sont nets ;
- [ ] les trois familles de polices et leur hiérarchie sont perceptibles ;
- [ ] l'accent n'apparaît que pour une action ou un état significatif ;
- [ ] sidebar 248 px, topbar 64 px et liste 430 px sont respectées sur grand écran ;
- [ ] la transition à 1100 px recouvre la liste au lieu de l'écraser ;
- [ ] la transition à 760 px transforme la navigation en tiroir ;
- [ ] les états survolé, actif, focus et désactivé sont distinguables ;
- [ ] les popovers, toasts et panneaux ont la même élévation retenue ;
- [ ] le mouvement est court, utile et désactivable ;
- [ ] le thème clair n'est pas un simple inversement : il utilise les tokens clairs ci-dessus.

## 10. Écarts interdits

Ne pas introduire les choix suivants dans une application qui reprend MIRA :

- cartes blanches flottantes avec rayon 8–16 px ;
- dégradés hors logo, ombres diffuses partout ou fond flou ;
- boutons « pilule », badges très arrondis ou avatars circulaires ;
- plusieurs couleurs de marque concurrentes ;
- texte dense entièrement en Anton ou en JetBrains Mono ;
- grandes illustrations là où une icône trait et une phrase suffisent ;
- navigation mobile qui compresse les trois colonnes au lieu de passer au tiroir
  et à la pile de détail ;
- animation lente, rebondissante ou décorative ;
- dépendance à une police ou une icône téléchargée à l'exécution.

Le résultat recherché est sévère dans sa structure, vivant dans sa typographie,
et suffisamment silencieux pour que les données métier soient toujours le
premier plan.
