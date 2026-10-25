import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/engine/engine_mail_source.dart';
import 'package:mira/state/store.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/composer.dart';
import 'package:provider/provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'engine_source_test.dart' show FakeEngine;
import 'fake_template_engine.dart';

/// Moteur factice complet dont le domaine `template` est tenu par un faux dédié.
class RoutedEngine implements EngineClient {
  final FakeEngine base = FakeEngine();
  final FakeTemplateEngine templates = FakeTemplateEngine();

  @override
  Future<Map<String, dynamic>> call(Map<String, dynamic> request) =>
      request['domain'] == 'template'
      ? templates.call(request)
      : base.call(request);

  @override
  Stream<Map<String, dynamic>> events({int? after}) =>
      base.events(after: after);

  @override
  Future<String?> originalHtml(String messageId) =>
      base.originalHtml(messageId);

  @override
  Future<void> close() => base.close();
}

Future<(MailStore, RoutedEngine)> pumpComposer(WidgetTester tester) async {
  SharedPreferences.setMockInitialValues({});
  final prefs = await SharedPreferences.getInstance();
  final engine = RoutedEngine();
  engine.templates.templates.add({
    'id': 't1',
    'name': 'Salut',
    'subject': null,
    'body': 'Bonjour à tous',
    'created_at': 1,
    'updated_at': 1,
  });
  late final MailStore store;
  await tester.runAsync(() async {
    store = MailStore(prefs, source: await EngineMailSource.load(engine));
  });
  store.openCompose(const ComposeInit(mode: 'new', text: ''));
  await tester.binding.setSurfaceSize(const Size(1200, 900));
  addTearDown(() => tester.binding.setSurfaceSize(null));
  await tester.pumpWidget(
    ChangeNotifierProvider.value(
      value: store,
      child: MaterialApp(
        theme: buildTheme(false),
        home: Scaffold(body: Composer(store.c!)),
      ),
    ),
  );
  await tester.pumpAndSettle();
  return (store, engine);
}

void main() {
  testWidgets('le bouton « Insérer un modèle » ouvre le sélecteur', (t) async {
    final (store, _) = await pumpComposer(t);
    await t.tap(find.byKey(const ValueKey('insert-template')));
    await t.pumpAndSettle();
    expect(find.byKey(const ValueKey('picker-row-t1')), findsOneWidget);
    await t.tap(find.byKey(const ValueKey('picker-row-t1')));
    await t.pumpAndSettle();
    expect(store.c!.body.text, contains('Bonjour à tous'));
    store.dispose();
  });

  testWidgets('taper « / » après un espace ouvre le sélecteur et le remplace', (
    t,
  ) async {
    final (store, _) = await pumpComposer(t);
    final body = find.byType(TextField).last;
    await t.enterText(body, 'Salut ');
    await t.enterText(body, 'Salut /');
    await t.pumpAndSettle();
    expect(find.byKey(const ValueKey('picker-row-t1')), findsOneWidget);
    await t.tap(find.byKey(const ValueKey('picker-row-t1')));
    await t.pumpAndSettle();
    expect(store.c!.body.text, 'Salut Bonjour à tous');
    store.dispose();
  });

  testWidgets('un caractère tapé avant l’ouverture du sélecteur est conservé', (
    t,
  ) async {
    final (store, _) = await pumpComposer(t);
    final body = find.byType(TextField).last;
    await t.enterText(body, 'Salut ');
    await t.enterText(body, 'Salut /');
    await t.enterText(body, 'Salut /m');
    await t.pumpAndSettle();
    await t.tap(find.byKey(const ValueKey('picker-row-t1')));
    await t.pumpAndSettle();
    expect(store.c!.body.text, 'Salut Bonjour à tousm');
    store.dispose();
  });

  testWidgets('une barre oblique dans une adresse n’ouvre rien', (t) async {
    final (store, _) = await pumpComposer(t);
    final body = find.byType(TextField).last;
    await t.enterText(body, 'http:/');
    await t.enterText(body, 'http://');
    await t.pumpAndSettle();
    expect(find.byKey(const ValueKey('picker-row-t1')), findsNothing);
    store.dispose();
  });

  testWidgets('Échap ferme le sélecteur ouvert par « / » sans rien changer', (
    t,
  ) async {
    final (store, _) = await pumpComposer(t);
    final body = find.byType(TextField).last;
    await t.enterText(body, 'A ');
    await t.enterText(body, 'A /');
    await t.pumpAndSettle();
    await t.sendKeyEvent(LogicalKeyboardKey.escape);
    await t.pumpAndSettle();
    expect(store.c!.body.text, 'A /');
    store.dispose();
  });
}
