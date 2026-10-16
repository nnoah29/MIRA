import 'package:flutter/widgets.dart';

import 'en.dart';

/// Langues de l'interface. Le français est la langue source : chaque texte de l'application
/// est écrit en français et sert de clé de traduction (`t('Texte')`).
enum AppLang {
  fr('fr', 'fr_FR', 'Français'),
  en('en', 'en_US', 'English');

  final String code, dateLocale, label;
  const AppLang(this.code, this.dateLocale, this.label);

  Locale get locale => Locale(code);

  static AppLang fromCode(String? c) =>
      AppLang.values.firstWhere((l) => l.code == c, orElse: () => AppLang.fr);
}

/// Langue active (modifiée par `MailStore.setLang`).
AppLang appLang = AppLang.fr;

/// Traduit [fr] dans la langue active. Les `{0}`, `{1}`… sont remplacés par [args].
String t(String fr, [List<Object?> args = const []]) {
  var s = appLang == AppLang.fr ? fr : (enStrings[fr] ?? fr);
  for (var i = 0; i < args.length; i++) {
    s = s.replaceAll('{$i}', '${args[i]}');
  }
  return s;
}

/// « 1 message », « 3 messages » : [one] et [other] sont des clés de traduction.
String tn(int n, String one, String other) => '$n ${t(n > 1 ? other : one)}';
