import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/engine/rule_admin.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/rules_settings.dart';

/// Moteur factice des règles : liste vide, export fixe, import programmable.
class _Engine implements EngineClient {
  final calls = <Map<String, dynamic>>[];
  EngineException? importFails;

  @override
  Future<Map<String, dynamic>> call(Map<String, dynamic> request) async {
    calls.add(request);
    final op = (request['request'] as Map)['op'];
    final data = switch (op) {
      'list_rules' => {'type': 'rules', 'data': []},
      'list_rule_catalog' => {
        'type': 'catalog',
        'data': {'conditions': [], 'actions': []},
      },
      'export_rules' => {
        'type': 'rules_export',
        'data': {'yaml': 'mira_export: {}\n'},
      },
      'import_rules' => _imported(),
      _ => {'type': 'done'},
    };
    return {'type': 'rule', 'data': data};
  }

  Map<String, dynamic> _imported() {
    final failure = importFails;
    if (failure != null) throw failure;
    return {
      'type': 'rules_import',
      'data': {
        'entries': [
          {
            'position': 1,
            'name': 'Lettres',
            'outcome': {'type': 'imported'},
          },
        ],
      },
    };
  }

  @override
  Stream<Map<String, dynamic>> events({int? after}) => const Stream.empty();

  @override
  Future<String?> originalHtml(String messageId) async => null;

  @override
  Future<void> close() async {}
}

Future<_Engine> _pump(
  WidgetTester t, {
  required Future<Uri?> Function(String, Uint8List) saveFile,
  required Future<String?> Function() pickFile,
  EngineException? importFails,
}) async {
  final engine = _Engine()..importFails = importFails;
  await t.binding.setSurfaceSize(const Size(900, 1600));
  addTearDown(() => t.binding.setSurfaceSize(null));
  await t.pumpWidget(
    MaterialApp(
      theme: buildTheme(false),
      home: Scaffold(
        body: SingleChildScrollView(
          child: RulesSettings(
            admin: RuleAdmin(engine),
            accounts: const {'a1': 'a1@example.org'},
            changes: const Stream.empty(),
            saveFile: saveFile,
            pickFile: pickFile,
            clock: () => DateTime(2026, 10, 27),
          ),
        ),
      ),
    ),
  );
  await t.pumpAndSettle();
  return engine;
}

void main() {
  testWidgets('Exporter enregistre le texte du moteur sous un nom daté', (
    t,
  ) async {
    String? name;
    Uint8List? bytes;
    await _pump(
      t,
      saveFile: (n, b) async {
        name = n;
        bytes = b;
        return Uri.file('/tmp/$n');
      },
      pickFile: () async => null,
    );
    await t.tap(find.byKey(const ValueKey('export-rules')));
    await t.pumpAndSettle();
    expect(name, 'mira-regles-2026-10-27.yaml');
    expect(utf8.decode(bytes!), 'mira_export: {}\n');
  });

  testWidgets(
    'Importer lit le fichier, appelle le moteur et montre le rapport',
    (t) async {
      final engine = await _pump(
        t,
        saveFile: (n, b) async => null,
        pickFile: () async => 'mira_export: {}\n',
      );
      await t.tap(find.byKey(const ValueKey('import-rules')));
      await t.pumpAndSettle();
      await t.tap(find.byKey(const ValueKey('import-replace')));
      await t.tap(find.byKey(const ValueKey('import-confirm')));
      await t.pumpAndSettle();
      final call = engine.calls
          .map((c) => c['request'] as Map)
          .firstWhere((r) => r['op'] == 'import_rules');
      expect(call['yaml'], 'mira_export: {}\n');
      expect(call['replace_existing'], true);
      expect(find.textContaining('1 importée'), findsOneWidget);
    },
  );

  testWidgets('aucun fichier choisi : le moteur n’est pas appelé', (t) async {
    final engine = await _pump(
      t,
      saveFile: (n, b) async => null,
      pickFile: () async => null,
    );
    await t.tap(find.byKey(const ValueKey('import-rules')));
    await t.pumpAndSettle();
    expect(
      engine.calls.where((c) => (c['request'] as Map)['op'] == 'import_rules'),
      isEmpty,
    );
  });

  testWidgets('une erreur de syntaxe affiche ligne et colonne', (t) async {
    await _pump(
      t,
      saveFile: (n, b) async => null,
      pickFile: () async => 'x',
      importFails: const EngineException(
        'invalid_input.yaml_syntax',
        'YAML invalide à la ligne 3, colonne 5',
      ),
    );
    await t.tap(find.byKey(const ValueKey('import-rules')));
    await t.pumpAndSettle();
    await t.tap(find.byKey(const ValueKey('import-confirm')));
    await t.pumpAndSettle();
    expect(find.textContaining('Fichier YAML invalide'), findsOneWidget);
    expect(find.textContaining('ligne 3, colonne 5'), findsOneWidget);
  });
}
