import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/template_admin.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/template_picker.dart';

const list = [
  EngineTemplate('t1', 'Merci', null, 'a'),
  EngineTemplate('t2', 'Relance client', 'Objet', 'b'),
  EngineTemplate('t3', 'Remerciement', null, 'c'),
];

Future<List<EngineTemplate?>> pumpPicker(WidgetTester tester) async {
  final results = <EngineTemplate?>[];
  await tester.pumpWidget(
    MaterialApp(
      theme: buildTheme(false),
      home: Scaffold(
        body: Builder(
          builder: (context) => TextButton(
            key: const ValueKey('open'),
            onPressed: () async =>
                results.add(await showTemplatePicker(context, list)),
            child: const Text('ouvrir'),
          ),
        ),
      ),
    ),
  );
  await tester.tap(find.byKey(const ValueKey('open')));
  await tester.pumpAndSettle();
  return results;
}

void main() {
  testWidgets('liste tous les modèles à l’ouverture', (t) async {
    await pumpPicker(t);
    for (final item in list) {
      expect(find.byKey(ValueKey('picker-row-${item.id}')), findsOneWidget);
    }
  });

  testWidgets('la saisie filtre par nom sans tenir compte de la casse', (
    t,
  ) async {
    await pumpPicker(t);
    await t.enterText(find.byKey(const ValueKey('template-search')), 'RE');
    await t.pumpAndSettle();
    expect(find.byKey(const ValueKey('picker-row-t1')), findsNothing);
    expect(find.byKey(const ValueKey('picker-row-t2')), findsOneWidget);
    expect(find.byKey(const ValueKey('picker-row-t3')), findsOneWidget);
  });

  testWidgets('Entrée choisit le premier modèle de la liste filtrée', (
    t,
  ) async {
    final results = await pumpPicker(t);
    await t.enterText(find.byKey(const ValueKey('template-search')), 'rem');
    await t.testTextInput.receiveAction(TextInputAction.done);
    await t.pumpAndSettle();
    expect(results.single?.id, 't3');
  });

  testWidgets('toucher une ligne la choisit', (t) async {
    final results = await pumpPicker(t);
    await t.tap(find.byKey(const ValueKey('picker-row-t2')));
    await t.pumpAndSettle();
    expect(results.single?.id, 't2');
  });

  testWidgets('Échap ferme sans rien choisir', (t) async {
    final results = await pumpPicker(t);
    await t.sendKeyEvent(LogicalKeyboardKey.escape);
    await t.pumpAndSettle();
    expect(results, [null]);
    expect(find.byKey(const ValueKey('template-search')), findsNothing);
  });

  testWidgets('aucun résultat : message et Entrée sans effet', (t) async {
    final results = await pumpPicker(t);
    await t.enterText(find.byKey(const ValueKey('template-search')), 'zzz');
    await t.testTextInput.receiveAction(TextInputAction.done);
    await t.pumpAndSettle();
    expect(find.text('Aucun modèle trouvé.'), findsOneWidget);
    expect(results, isEmpty);
  });
}
