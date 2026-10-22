import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/data/rule_models.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/rule_labels.dart';
import 'package:mira/ui/rule_params.dart';

/// Saisit `text` dans le filtre d'un test « objet » d'opérateur `op` et rend les valeurs enregistrées.
Future<List<dynamic>> typeValues(WidgetTester t, String op, String text) async {
  final data = <String, dynamic>{
    'type': 'subject',
    'matcher': {'op': op, 'values': <dynamic>[]},
  };
  await t.pumpWidget(
    MaterialApp(
      theme: buildTheme(false),
      home: Scaffold(
        body: RuleParam(
          keyPrefix: 'c',
          name: 'matcher',
          data: data,
          onChanged: () {},
        ),
      ),
    ),
  );
  await t.enterText(find.byType(TextFormField), text);
  await t.pump();
  return (data['matcher'] as Map)['values'] as List;
}

/// Monte le paramètre `name` avec les choix donnés et rend la donnée.
Future<Map<String, dynamic>> pumpChoice(
  WidgetTester t,
  String name,
  String current,
  List<RuleChoice> choices,
) async {
  final data = <String, dynamic>{'type': 'move_to', name: current};
  await t.pumpWidget(
    MaterialApp(
      theme: buildTheme(false),
      home: Scaffold(
        body: RuleParam(
          keyPrefix: 'a',
          name: name,
          data: data,
          options: {name: choices},
          onChanged: () {},
        ),
      ),
    ),
  );
  return data;
}

void main() {
  testWidgets('hors « dans la liste », la virgule fait partie de la valeur', (
    t,
  ) async {
    expect(await typeValues(t, 'contains', 'Doe, John'), ['Doe, John']);
    expect(await typeValues(t, 'regex', 'a{1,3}'), ['a{1,3}']);
  });

  testWidgets('« dans la liste » découpe sur la virgule et retire les vides', (
    t,
  ) async {
    expect(await typeValues(t, 'in_list', 'a, b, '), ['a', 'b']);
    expect(await typeValues(t, 'in_list', ' , ,'), isEmpty);
  });

  testWidgets('une saisie vide ou faite d\'espaces ne donne aucune valeur', (
    t,
  ) async {
    expect(await typeValues(t, 'contains', '   '), isEmpty);
  });

  test('un filtre neuf n\'a aucune valeur vide', () {
    final matcher = defaultParam('matcher') as Map;
    expect(matcher['values'], isEmpty);
  });

  testWidgets('une boîte se choisit dans la liste proposée', (t) async {
    final data = await pumpChoice(t, 'mailbox', '', const [
      RuleChoice('mb-1', 'Factures'),
      RuleChoice('mb-2', 'Perso'),
    ]);
    await t.tap(find.byKey(const ValueKey('a-mailbox')));
    await t.pumpAndSettle();
    await t.tap(find.text('Perso').last);
    await t.pumpAndSettle();
    expect(data['mailbox'], 'mb-2');
  });

  testWidgets('une boîte disparue reste affichée et signalée', (t) async {
    await pumpChoice(t, 'mailbox', 'mb-gone', const [
      RuleChoice('mb-1', 'Factures'),
    ]);
    expect(find.text('mb-gone (introuvable)'), findsOneWidget);
  });
}
