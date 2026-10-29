import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/ai_admin.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/ai_settings.dart';
import 'package:mira/ui/overlays.dart';

import 'fake_ai_engine.dart';

Map<String, dynamic> provider(String id, String name, {bool free = false}) => {
  'id': id,
  'display_name': name,
  'base_url': 'https://example.org/v1',
  'base_url_editable': free,
};

Map<String, dynamic> connection(
  String id, {
  String provider = 'openai',
  String? label,
  bool isDefault = false,
  bool hasKey = false,
  String? model,
  String? baseUrl,
  List<String>? models,
}) => {
  'id': id,
  'provider': provider,
  'label': label ?? 'Connexion $id',
  'base_url': baseUrl,
  'default_model': model,
  'is_default': isDefault,
  'has_key': hasKey,
  'consent_at': null,
  'models': models == null
      ? null
      : {
          'models': [
            for (final m in models) {'id': m, 'created_at': 1},
          ],
          'fetched_at': 1,
        },
};

Future<FakeAiEngine> pumpSettings(
  WidgetTester tester, {
  List<Map<String, dynamic>> connections = const [],
}) async {
  final engine = FakeAiEngine()
    ..providers.addAll([
      provider('openai', 'OpenAI'),
      provider('anthropic', 'Anthropic'),
      provider('generic', 'Compatible OpenAI', free: true),
    ])
    ..connections.addAll(connections);
  await tester.binding.setSurfaceSize(const Size(900, 2400));
  addTearDown(() => tester.binding.setSurfaceSize(null));
  addTearDown(Pops.close);
  await tester.pumpWidget(
    MaterialApp(
      theme: buildTheme(false),
      home: Scaffold(
        body: SingleChildScrollView(child: AiSettings(admin: AiAdmin(engine))),
      ),
    ),
  );
  await tester.pumpAndSettle();
  return engine;
}

String fieldText(WidgetTester tester, String key) =>
    tester.widget<TextField>(find.byKey(ValueKey(key))).controller!.text;

Finder byKey(String key) => find.byKey(ValueKey(key));

Future<void> tapKey(WidgetTester tester, String key) async {
  await tester.tap(byKey(key));
  await tester.pumpAndSettle();
}

String modelButtonText(WidgetTester tester, String id) => tester
    .widget<Text>(
      find.descendant(
        of: byKey('ai-model-button-$id'),
        matching: find.byType(Text),
      ),
    )
    .data!;
