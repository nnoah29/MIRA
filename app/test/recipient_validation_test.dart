import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'composer_wiring_test.dart' show pumpComposer;

Future<void> typeRecipient(WidgetTester t, String text) async {
  await t.tap(find.byType(TextField).first);
  await t.enterText(find.byType(TextField).first, text);
  await t.sendKeyEvent(LogicalKeyboardKey.enter);
  await t.pumpAndSettle();
}

void main() {
  testWidgets('une adresse que le moteur refuserait n\'est pas ajoutée', (
    t,
  ) async {
    final (store, _) = await pumpComposer(t);
    for (final a in ['bob@example.org.', 'bob@a..org', 'bob@.example.org']) {
      await typeRecipient(t, a);
      expect(store.c!.to, isEmpty, reason: a);
    }
    store.dispose();
  });

  testWidgets('une adresse valide est ajoutée', (t) async {
    final (store, _) = await pumpComposer(t);
    await typeRecipient(t, 'bob@example.org');
    expect(store.c!.to.map((p) => p.email), ['bob@example.org']);
    store.dispose();
  });
}
