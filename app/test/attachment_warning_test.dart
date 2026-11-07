import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/data/models.dart';

import 'composer_wiring_test.dart' show pumpComposer;

Finder byKey(String key) => find.byKey(ValueKey(key));

Future<void> settle(WidgetTester t) async {
  for (var i = 0; i < 20; i++) {
    await t.pump(const Duration(milliseconds: 20));
  }
}

void main() {
  testWidgets('un avertissement s\'affiche, « Envoyer quand même » envoie', (
    t,
  ) async {
    final (store, engine) = await pumpComposer(t);
    store.c!.to.add(const Person('Bob', 'bob@example.org'));
    store.c!.body.setContent('Voir le devis ci-joint', List.filled(22, 0));
    expect(byKey('attach-warning'), findsNothing);
    await t.tap(byKey('composer-send'));
    await t.pumpAndSettle();
    expect(byKey('attach-warning'), findsOneWidget);
    expect(store.c, isNotNull);
    await t.tap(byKey('attach-warning-send'));
    await settle(t);
    expect(store.c, isNull);
    expect(
      engine.base.composed.where((c) => c['op'] == 'send_draft'),
      isNotEmpty,
    );
    store.dispose();
  });

  testWidgets('modifier le message fait disparaître l\'avertissement', (
    t,
  ) async {
    final (store, _) = await pumpComposer(t);
    store.c!.to.add(const Person('Bob', 'bob@example.org'));
    store.c!.body.setContent('Voir le devis ci-joint', List.filled(22, 0));
    await t.tap(byKey('composer-send'));
    await t.pumpAndSettle();
    expect(byKey('attach-warning'), findsOneWidget);
    store.touch();
    await t.pumpAndSettle();
    expect(byKey('attach-warning'), findsNothing);
    store.dispose();
  });

  testWidgets('un message sans mention part sans avertissement', (t) async {
    final (store, _) = await pumpComposer(t);
    store.c!.to.add(const Person('Bob', 'bob@example.org'));
    store.c!.body.setContent('Bonjour Bob', List.filled(11, 0));
    await t.tap(byKey('composer-send'));
    await settle(t);
    expect(byKey('attach-warning'), findsNothing);
    expect(store.c, isNull);
    store.dispose();
  });
}
