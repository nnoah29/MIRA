import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/i18n/engine_errors.dart';
import 'package:mira/i18n/i18n.dart';

/// Codes d'erreur déclarés par le moteur (`ErrorCode("…")` du vocabulaire).
Set<String> engineErrorCodes() {
  final dir = Directory('../engine/crates/mira-vocabulary/src/error/codes');
  final code = RegExp(r'ErrorCode\("([a-z_.]+)"\)');
  return {
    for (final f in dir.listSync().whereType<File>().where(
      (f) => f.path.endsWith('.rs'),
    ))
      for (final m in code.allMatches(f.readAsStringSync())) m.group(1)!,
  };
}

void main() {
  tearDown(() => appLang = AppLang.fr);

  test('chaque code d’erreur du moteur a un texte, et seulement eux', () {
    final declared = engineErrorCodes();
    expect(declared.length, greaterThan(50));
    expect(declared.difference(engineErrorTexts.keys.toSet()), isEmpty);
    expect(engineErrorTexts.keys.toSet().difference(declared), isEmpty);
  });

  test('une erreur du moteur est dite dans la langue active, jamais en français par défaut', () {
    final error = EngineException(
      'server.quota_exceeded',
      'Quota de la boîte dépassé',
    );
    expect(engineErrorText(error), 'Boîte pleine sur le serveur');
    appLang = AppLang.en;
    expect(engineErrorText(error), 'Mailbox full on the server');
    expect(engineErrorText(error), isNot(contains('Quota')));
  });

  test('un code inconnu donne le code, un message de l’app reste tel quel', () {
    appLang = AppLang.en;
    final future = EngineException('futur.code', 'Message français du moteur');
    expect(engineErrorText(future), 'Error: futur.code');
    final own = EngineException.local(
      'invalid_input.parameter',
      'Port invalid (1 to 65535)',
    );
    expect(engineErrorText(own), 'Port invalid (1 to 65535)');
    expect(engineErrorText(StateError('x')), contains('Unexpected error'));
  });

  test('un texte non traduit passe à la langue de repli, puis au français', () {
    String? fallback(String s) => s == 'a' ? 'repli-a' : null;
    expect(resolveText('a', {'a': 'propre'}, fallback), 'propre');
    expect(resolveText('a', {'a': ''}, fallback), 'repli-a');
    expect(resolveText('a', const {}, fallback), 'repli-a');
    expect(resolveText('b', const {}, fallback), isNull);
    expect(resolveText('b', const {}, null), isNull);
  });

  test('chaque langue a un code, une locale de dates et un nom uniques', () {
    for (final pick in <String Function(AppLang)>[
      (l) => l.code,
      (l) => l.dateLocale,
      (l) => l.label,
    ]) {
      final values = AppLang.values.map(pick).toList();
      expect(values.toSet().length, values.length);
    }
    for (final l in AppLang.values) {
      expect(
        l.fallbackCode == null ||
            AppLang.values.any((o) => o.code == l.fallbackCode),
        isTrue,
      );
    }
  });

  test(
    'audit : chaque texte utilisé est traduit dans chaque langue, aucun texte français en dur',
    () {
      final run = Process.runSync('python3', ['../tools/i18n-audit', '--json']);
      final report = jsonDecode(run.stdout as String) as Map<String, dynamic>;
      final langs = report['languages'] as Map<String, dynamic>;
      for (final entry in langs.entries) {
        final r = entry.value as Map<String, dynamic>;
        expect(
          r['missing'],
          isEmpty,
          reason: '${entry.key} : textes sans traduction',
        );
        expect(
          r['placeholders'],
          isEmpty,
          reason: '${entry.key} : paramètres {n} incohérents',
        );
        expect(
          r['unused'],
          isEmpty,
          reason: '${entry.key} : traductions inutilisées',
        );
      }
      expect(
        report['hardcoded'],
        isEmpty,
        reason: 'texte français écrit en dur',
      );
      expect(
        langs.keys,
        containsAll(
          AppLang.values.where((l) => l.strings.isNotEmpty).map((l) => l.code),
        ),
      );
    },
    skip: Process.runSync('python3', ['--version']).exitCode != 0
        ? 'python3 absent'
        : null,
  );
}
