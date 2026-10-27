import 'dart:async';
import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/engine/engine_mail_source.dart';
import 'package:mira/state/store.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/labels_settings.dart';
import 'package:provider/provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'engine_source_test.dart' show FakeEngine;

/// Faux moteur qui sait exporter et importer un fichier de libellés.
class _LabelFileEngine extends FakeEngine {
  final asked = <Map<String, dynamic>>[];
  Completer<void>? hold;
  EngineException? failImport;

  @override
  Future<Map<String, dynamic>> call(Map<String, dynamic> request) async {
    final r = request['request'] as Map<String, dynamic>;
    if (request['domain'] != 'local_label' ||
        !'${r['op']}'.endsWith('_local_labels_file')) {
      return super.call(request);
    }
    asked.add(r);
    if (r['op'] == 'export_local_labels_file') {
      return {
        'type': 'local_label',
        'data': {
          'type': 'labels_file_export',
          'data': {'yaml': 'mira_export: {}\n'},
        },
      };
    }
    await hold?.future;
    if (failImport != null) throw failImport!;
    return {
      'type': 'local_label',
      'data': {
        'type': 'labels_file_import',
        'data': {
          'entries': [
            {
              'position': 1,
              'name': 'Compta',
              'outcome': {'type': 'imported'},
            },
          ],
        },
      },
    };
  }
}

Future<_LabelFileEngine> _pump(
  WidgetTester t, {
  required Future<Uri?> Function(String, Uint8List) saveFile,
  required Future<String?> Function() pickFile,
  EngineException? importFails,
  Completer<void>? hold,
}) async {
  SharedPreferences.setMockInitialValues({});
  final prefs = await SharedPreferences.getInstance();
  final fake = _LabelFileEngine()
    ..failImport = importFails
    ..hold = hold;
  final store = MailStore(prefs, source: await EngineMailSource.load(fake));
  await t.binding.setSurfaceSize(const Size(1000, 1800));
  addTearDown(() => t.binding.setSurfaceSize(null));
  await t.pumpWidget(
    ChangeNotifierProvider<MailStore>.value(
      value: store,
      child: MaterialApp(
        theme: buildTheme(false),
        home: Scaffold(
          body: SingleChildScrollView(
            child: LabelsSettings(
              saveFile: saveFile,
              pickFile: pickFile,
              clock: () => DateTime(2026, 10, 27),
            ),
          ),
        ),
      ),
    ),
  );
  await t.pumpAndSettle();
  return fake;
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
    await t.tap(find.byKey(const ValueKey('export-labels')));
    await t.pumpAndSettle();
    expect(name, 'mira-libelles-2026-10-27.yaml');
    expect(utf8.decode(bytes!), 'mira_export: {}\n');
  });

  testWidgets(
    'Importer lit le fichier, demande l’option, appelle le moteur et montre le rapport',
    (t) async {
      final fake = await _pump(
        t,
        saveFile: (n, b) async => null,
        pickFile: () async => 'mira_export: {}\n',
      );
      await t.tap(find.byKey(const ValueKey('import-labels')));
      await t.pumpAndSettle();
      expect(find.text('Remplacer les libellés existants'), findsOneWidget);
      await t.tap(find.byKey(const ValueKey('import-replace')));
      await t.tap(find.byKey(const ValueKey('import-confirm')));
      await t.pumpAndSettle();
      final call = fake.asked.firstWhere(
        (r) => r['op'] == 'import_local_labels_file',
      );
      expect(call['yaml'], 'mira_export: {}\n');
      expect(call['replace_existing'], true);
      expect(find.textContaining('1 importé'), findsOneWidget);
    },
  );

  testWidgets('aucun fichier choisi : le moteur n’est pas appelé', (t) async {
    final fake = await _pump(
      t,
      saveFile: (n, b) async => null,
      pickFile: () async => null,
    );
    await t.tap(find.byKey(const ValueKey('import-labels')));
    await t.pumpAndSettle();
    expect(fake.asked, isEmpty);
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
    await t.tap(find.byKey(const ValueKey('import-labels')));
    await t.pumpAndSettle();
    await t.tap(find.byKey(const ValueKey('import-confirm')));
    await t.pumpAndSettle();
    expect(find.textContaining('Fichier YAML invalide'), findsOneWidget);
    expect(find.textContaining('ligne 3, colonne 5'), findsOneWidget);
  });

  testWidgets('pendant l’import, les deux boutons sont désactivés', (t) async {
    final hold = Completer<void>();
    final fake = await _pump(
      t,
      saveFile: (n, b) async => null,
      pickFile: () async => 'mira_export: {}\n',
      hold: hold,
    );
    await t.tap(find.byKey(const ValueKey('import-labels')));
    await t.pumpAndSettle();
    await t.tap(find.byKey(const ValueKey('import-confirm')));
    await t.pump();
    await t.tap(
      find.byKey(const ValueKey('import-labels')),
      warnIfMissed: false,
    );
    await t.tap(
      find.byKey(const ValueKey('export-labels')),
      warnIfMissed: false,
    );
    await t.pump();
    expect(
      fake.asked.where((r) => r['op'] == 'import_local_labels_file').length,
      1,
    );
    expect(
      fake.asked.where((r) => r['op'] == 'export_local_labels_file'),
      isEmpty,
    );
    hold.complete();
    await t.pumpAndSettle();
  });
}
