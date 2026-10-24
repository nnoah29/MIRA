import 'package:flutter/widgets.dart';

import 'lang/en.dart';

/// Langues de l'interface. Le français est la langue source : chaque texte de l'application
/// est écrit en français et sert de clé de traduction (`t('Texte')`).
///
/// Ajouter une langue : voir `docs/I18N.md` (un fichier `lang/xx.dart`, une ligne ici).
/// Règle de pluriel d'une langue : donne la catégorie (`one`, `few`, `many`, `other`…, noms du CLDR) d'un nombre.
typedef PluralRule = String Function(int n);

/// Français : 0 et 1 sont au singulier.
String pluralFr(int n) => n == 0 || n == 1 ? 'one' : 'other';

/// Anglais : seul 1 est au singulier.
String pluralEn(int n) => n == 1 ? 'one' : 'other';

enum AppLang {
  fr('fr', 'fr_FR', 'Français', {}, null, pluralFr), // i18n:ignore
  en('en', 'en_US', 'English', enStrings, null, pluralEn);

  /// Code de la langue, locale des dates, nom affiché dans le sélecteur.
  final String code, dateLocale, label;

  /// Traductions des textes français ; vide pour le français lui-même.
  final Map<String, String> strings;

  /// Code de la langue de repli pour un texte non traduit (`null` : le texte français).
  final String? fallbackCode;

  /// Catégorie de pluriel d'un nombre dans cette langue.
  final PluralRule plural;

  const AppLang(
    this.code,
    this.dateLocale,
    this.label,
    this.strings,
    this.fallbackCode,
    this.plural,
  );

  Locale get locale => Locale(code);

  /// Langue de repli : à défaut de traduction, l'interface montre cette langue avant le français.
  AppLang? get fallback => fallbackCode == null
      ? null
      : AppLang.values.firstWhere((l) => l.code == fallbackCode);

  /// Traduction du texte français [fr] dans cette langue, ou `null` s'il n'y en a pas.
  String? lookup(String fr) => resolveText(fr, strings, fallback?.lookup);

  static AppLang fromCode(String? c) =>
      AppLang.values.firstWhere((l) => l.code == c, orElse: () => AppLang.fr);
}

/// Traduction de [fr] dans [table] ; un texte absent ou vide passe à [fallback] (`null` : pas de traduction).
String? resolveText(
  String fr,
  Map<String, String> table,
  String? Function(String)? fallback,
) {
  final own = table[fr];
  if (own != null && own.isNotEmpty) return own;
  return fallback?.call(fr);
}

/// Langue active (modifiée par `MailStore.setLang`).
AppLang appLang = AppLang.fr;

/// Traduit [fr] dans la langue active. Les `{0}`, `{1}`… sont remplacés par [args].
String t(String fr, [List<Object?> args = const []]) {
  var s = appLang == AppLang.fr ? fr : (appLang.lookup(fr) ?? fr);
  for (var i = 0; i < args.length; i++) {
    s = s.replaceAll('{$i}', '${args[i]}');
  }
  return s;
}

/// « 1 message », « 3 messages » : [one] et [other] sont des clés de traduction. Une langue qui a d'autres
/// formes de pluriel les écrit sous la clé `<other>#<catégorie>` (par exemple `messages#few`) ; sans elle, [other].
String tn(int n, String one, String other) =>
    '$n ${pluralText(appLang.plural(n), one, other, t, appLang.lookup)}';

/// Texte de la forme de pluriel [category] : `one` ou `other` par [translate], sinon la forme propre à la langue.
String pluralText(
  String category,
  String one,
  String other,
  String Function(String) translate,
  String? Function(String) lookup,
) {
  if (category == 'one') return translate(one);
  if (category == 'other') return translate(other);
  return lookup('$other#$category') ?? translate(other);
}

/// Marque un texte français à traduire plus tard (données, constantes) : rend le texte tel quel,
/// l'audit des traductions le compte comme une clé. À afficher ensuite avec `t(valeur)`.
String tkey(String fr) => fr;
