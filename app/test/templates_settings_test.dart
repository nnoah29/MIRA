import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/engine/template_admin.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/templates_settings.dart';

import 'fake_template_engine.dart';

Map<String, dynamic> stored(String id, String name, String body) => {
  'id': id,
  'name': name,
  'subject': null,
  'body': body,
  'created_at': 1,
  'updated_at': 1,
};

Future<FakeTemplateEngine> pumpSettings(
  WidgetTester tester, {
  List<Map<String, dynamic>> templates = const [],
}) async {
  final engine = FakeTemplateEngine()..templates.addAll(templates);
  await tester.binding.setSurfaceSize(const Size(900, 1600));
  addTearDown(() => tester.binding.setSurfaceSize(null));
  await tester.pumpWidget(
    MaterialApp(
      theme: buildTheme(false),
      home: Scaffold(
        body: SingleChildScrollView(
          child: TemplatesSettings(
            admin: TemplateAdmin(engine),
            changes: const Stream<void>.empty(),
          ),
        ),
      ),
    ),
  );
  await tester.pumpAndSettle();
  return engine;
}

void main() {
  testWidgets('liste les modèles par nom', (t) async {
    await pumpSettings(
      t,
      templates: [stored('t1', 'Merci', 'a'), stored('t2', 'Relance', 'b')],
    );
    expect(find.text('Merci'), findsOneWidget);
    expect(find.text('Relance'), findsOneWidget);
  });

  testWidgets('créer un modèle avec une variable insérée au curseur', (
    t,
  ) async {
    final engine = await pumpSettings(t);
    await t.tap(find.byKey(const ValueKey('add-template')));
    await t.pumpAndSettle();
    await t.enterText(find.byKey(const ValueKey('template-name')), 'Salut');
    await t.enterText(find.byKey(const ValueKey('template-body')), 'Bonjour ');
    await t.tap(find.byKey(const ValueKey('template-var-contact.nom')));
    await t.pumpAndSettle();
    await t.tap(find.byKey(const ValueKey('save-template')));
    await t.pumpAndSettle();
    final sent = engine.requestsOf('create_template').single;
    expect(sent['name'], 'Salut');
    expect(sent['body'], 'Bonjour {{contact.nom}}');
    expect(sent['subject'], isNull);
    expect(find.byKey(const ValueKey('template-name')), findsNothing);
    expect(find.text('Salut'), findsOneWidget);
  });

  testWidgets('modifier un modèle envoie update_template', (t) async {
    final engine = await pumpSettings(
      t,
      templates: [stored('t1', 'Merci', 'Corps')],
    );
    await t.tap(find.byKey(const ValueKey('template-row-t1')));
    await t.pumpAndSettle();
    await t.enterText(find.byKey(const ValueKey('template-name')), 'Merci 2');
    await t.tap(find.byKey(const ValueKey('save-template')));
    await t.pumpAndSettle();
    final sent = engine.requestsOf('update_template').single;
    expect(
      (sent['template'], sent['name'], sent['body']),
      ('t1', 'Merci 2', 'Corps'),
    );
  });

  testWidgets('supprimer demande confirmation puis envoie delete_template', (
    t,
  ) async {
    final engine = await pumpSettings(
      t,
      templates: [stored('t1', 'Merci', 'Corps')],
    );
    await t.tap(find.byKey(const ValueKey('template-delete-t1')));
    await t.pumpAndSettle();
    expect(engine.requestsOf('delete_template'), isEmpty);
    await t.tap(find.byKey(const ValueKey('confirm-delete-template')));
    await t.pumpAndSettle();
    expect(engine.requestsOf('delete_template').single['template'], 't1');
    expect(find.text('Merci'), findsNothing);
  });

  testWidgets('un nom déjà pris montre l’erreur traduite et garde l’éditeur', (
    t,
  ) async {
    final engine = await pumpSettings(t);
    await t.tap(find.byKey(const ValueKey('add-template')));
    await t.pumpAndSettle();
    await t.enterText(find.byKey(const ValueKey('template-name')), 'Merci');
    engine.failNext = const EngineException(
      'invalid_input.template_name_taken',
      'brut',
    );
    await t.tap(find.byKey(const ValueKey('save-template')));
    await t.pumpAndSettle();
    expect(find.text('Un modèle porte déjà ce nom'), findsOneWidget);
    expect(find.text('brut'), findsNothing);
    expect(find.byKey(const ValueKey('template-name')), findsOneWidget);
  });
}
