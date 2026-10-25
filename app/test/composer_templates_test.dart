import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/data/models.dart';
import 'package:mira/engine/template_admin.dart';
import 'package:mira/state/store.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/composer_templates.dart';

import 'fake_template_engine.dart';

class Harness {
  final FakeTemplateEngine engine = FakeTemplateEngine();
  final notes = <(String, bool)>[];
  int touched = 0;
  late ComposeState compose;
}

Future<Harness> pumpHarness(
  WidgetTester tester, {
  List<Person> to = const [Person('Léa Durand', 'lea@x.fr')],
  int? slashAt,
  String body = '',
  String subject = '',
}) async {
  final h = Harness();
  h.engine.templates.add({
    'id': 't1',
    'name': 'Salut',
    'subject': 'Pour {{contact.prénom}}',
    'body': 'Bonjour {{contact.nom}}, le {{date}}. {{moi.nom}}',
    'created_at': 1,
    'updated_at': 1,
  });
  h.compose = ComposeState(
    to: to,
    cc: [],
    bcc: [],
    atts: [],
    from: 'noah@x.fr',
  );
  h.compose.body.value = TextEditingValue(
    text: body,
    selection: TextSelection.collapsed(offset: body.length),
  );
  h.compose.subject.text = subject;
  await tester.pumpWidget(
    MaterialApp(
      theme: buildTheme(false),
      home: Scaffold(
        body: Builder(
          builder: (context) => TextButton(
            key: const ValueKey('open'),
            onPressed: () => pickAndInsertTemplate(
              context,
              c: h.compose,
              admin: TemplateAdmin(h.engine),
              fromName: 'Noah',
              slashAt: slashAt,
              now: DateTime(2026, 10, 25, 14, 30),
              notify: (message, {bool err = false}) =>
                  h.notes.add((message, err)),
              touch: () => h.touched++,
            ),
            child: const Text('ouvrir'),
          ),
        ),
      ),
    ),
  );
  await tester.tap(find.byKey(const ValueKey('open')));
  await tester.pumpAndSettle();
  return h;
}

Future<void> choose(WidgetTester tester) async {
  await tester.tap(find.byKey(const ValueKey('picker-row-t1')));
  await tester.pumpAndSettle();
}

void main() {
  testWidgets('le modèle choisi est rendu puis inséré au curseur', (t) async {
    final h = await pumpHarness(t, body: 'Début ');
    await choose(t);
    expect(
      h.compose.body.text,
      'Début Bonjour Léa Durand, le 25/10/2026. Noah',
    );
    expect(h.touched, 1);
    expect(h.notes, isEmpty);
    final sent = h.engine.requestsOf('render_template').single;
    expect((sent['context'] as Map)['account_email'], 'noah@x.fr');
  });

  testWidgets('l’objet du modèle remplit un champ objet vide seulement', (
    t,
  ) async {
    final empty = await pumpHarness(t);
    await choose(t);
    expect(empty.compose.subject.text, 'Pour Léa');
    final kept = await pumpHarness(t, subject: 'Mon objet');
    await choose(t);
    expect(kept.compose.subject.text, 'Mon objet');
  });

  testWidgets('la barre oblique déclencheuse est remplacée par le texte', (
    t,
  ) async {
    final h = await pumpHarness(t, body: 'Salut /', slashAt: 6);
    await choose(t);
    expect(
      h.compose.body.text,
      'Salut Bonjour Léa Durand, le 25/10/2026. Noah',
    );
  });

  testWidgets('les variables sans valeur restent et sont annoncées', (t) async {
    final h = await pumpHarness(t, to: const []);
    await choose(t);
    expect(h.compose.body.text, contains('{{contact.nom}}'));
    expect(h.notes.single, ('2 variables à compléter', false));
  });

  testWidgets(
    'seules les variables visibles dans le texte inséré sont comptées',
    (t) async {
      final h = await pumpHarness(
        t,
        to: const [Person('', 'lea@x.fr')],
        subject: 'Re: X',
      );
      await choose(t);
      expect(h.compose.subject.text, 'Re: X');
      expect(h.notes.single, ('1 variable à compléter', false));
    },
  );

  testWidgets('modèle supprimé entre-temps : erreur traduite, corps intact', (
    t,
  ) async {
    final h = await pumpHarness(t, body: 'Intact');
    h.engine.failNext = null;
    h.engine.templates.clear();
    await choose(t);
    expect(h.compose.body.text, 'Intact');
    expect(h.notes.single.$2, isTrue);
    expect(h.notes.single.$1, 'Modèle introuvable');
    expect(h.touched, 0);
  });

  testWidgets('Échap ferme le sélecteur sans rien insérer', (t) async {
    final h = await pumpHarness(t, body: 'Intact');
    await t.sendKeyEvent(LogicalKeyboardKey.escape);
    await t.pumpAndSettle();
    expect(h.compose.body.text, 'Intact');
    expect(h.engine.requestsOf('render_template'), isEmpty);
  });
}
