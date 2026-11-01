import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/state/store.dart';
import 'package:mira/ui/step_animator.dart';

import 'ai_panel_harness.dart';
import 'fake_ai_engine.dart';

Finder key(String k) => find.byKey(ValueKey(k));

Map<String, dynamic> proposal(Map<String, Object?> fields) => {
  'type': 'draft_proposed',
  'id': 'c1',
  'proposal': {
    'kind': 'edit',
    'to': null,
    'cc': null,
    'subject': null,
    'body': null,
    'selection': null,
    'reply_to': null,
    'identity': null,
    ...fields,
  },
};

List<Map<String, dynamic>> turn(
  List<Map<String, dynamic>> middle, {
  String text = '',
}) => [
  {'type': 'started', 'operation': 'op-9'},
  ...middle,
  if (text.isNotEmpty) {'type': 'text', 'text': text},
  {'type': 'done', 'reason': 'end_turn'},
];

Future<void> settle(WidgetTester tester) async {
  await tester.pump(const Duration(milliseconds: 300));
  await tester.pump(const Duration(milliseconds: 50));
}

/// Panneau ouvert avec une connexion d'IA, puis un nouveau message ouvert dans le composeur.
Future<(MailStore, FakeAiEngine)> withComposer(
  WidgetTester tester, {
  List<List<Map<String, dynamic>>> scripts = const [],
}) async {
  final engine = withDefaultConnection()..scripts.addAll(scripts);
  final store = await pumpPanel(tester, engine);
  await settle(tester);
  store.openCompose(const ComposeInit());
  await settle(tester);
  return (store, engine);
}

Future<void> ask(WidgetTester tester, String instruction) async {
  await tester.tap(key('composer-ai-open'));
  await settle(tester);
  await tester.enterText(key('composer-ai-input'), instruction);
  await tester.tap(key('composer-ai-send'));
  await settle(tester);
  await settle(tester);
}

void main() {
  setUpAll(() => StepAnimator.frameCap = null);

  testWidgets('button_is_hidden_without_an_ai_engine', (tester) async {
    final store = await pumpPanel(tester, null);
    store.openCompose(const ComposeInit());
    await settle(tester);
    expect(key('composer-ai-open'), findsNothing);
    await close(tester, store);
  });

  testWidgets('instruction_fills_the_draft_and_undo_restores_it', (
    tester,
  ) async {
    final (store, engine) = await withComposer(
      tester,
      scripts: [
        turn([
          proposal({
            'subject': 'Fin de stage',
            'to': [
              {'name': 'Mon boss', 'address': 'boss@example.org'},
            ],
            'body': 'Bonjour,\n\nMon stage se termine mercredi.',
          }),
        ], text: 'Voilà.'),
      ],
    );
    final before = store.c!.body.text;
    await ask(tester, 'dis à mon boss que mon stage finit mercredi');
    final c = store.c!;
    expect(c.subject.text, 'Fin de stage');
    expect(c.to.single.email, 'boss@example.org');
    expect(
      c.body.text,
      startsWith('Bonjour,\n\nMon stage se termine mercredi.'),
    );
    expect('Cordialement'.allMatches(c.body.text), hasLength(1));
    expect(key('composer-ai-undo'), findsOneWidget);
    expect(find.text('Voilà.'), findsOneWidget);
    expect(engine.chatStarts.single['mode'], {
      'kind': 'composer',
      'selection': false,
    });
    await tester.tap(key('composer-ai-undo'));
    await settle(tester);
    expect(c.body.text, before);
    expect(c.subject.text, '');
    expect(c.to, isEmpty);
    expect(key('composer-ai-undo'), findsNothing);
    await close(tester, store);
  });

  testWidgets('a_question_from_the_ai_is_shown_and_the_draft_is_untouched', (
    tester,
  ) async {
    final (store, _) = await withComposer(
      tester,
      scripts: [turn(const [], text: 'Quelle est son adresse ?')],
    );
    final before = store.c!.body.text;
    await ask(tester, 'écris à mon boss');
    expect(key('composer-ai-reply'), findsOneWidget);
    expect(find.text('Quelle est son adresse ?'), findsOneWidget);
    expect(store.c!.body.text, before);
    expect(key('composer-ai-undo'), findsNothing);
    await close(tester, store);
  });

  testWidgets('an_engine_error_is_shown_in_the_bar', (tester) async {
    final (store, _) = await withComposer(
      tester,
      scripts: [
        [
          {
            'type': 'error',
            'error': {
              'code': 'incompatible_state.ai_consent_required',
              'message': 'x',
            },
          },
        ],
      ],
    );
    await ask(tester, 'écris');
    expect(key('composer-ai-error'), findsOneWidget);
    expect(
      find.text('Autorise d’abord l’envoi de texte à ce service d’IA'),
      findsOneWidget,
    );
    await close(tester, store);
  });

  testWidgets('selection_actions_send_the_selected_passage_and_replace_it', (
    tester,
  ) async {
    final (store, engine) = await withComposer(
      tester,
      scripts: [
        turn([
          proposal({'selection': 'un grand merci'}),
        ]),
      ],
    );
    final c = store.c!;
    const text = 'Salut Marie, merci pour tout.\n\nCordialement';
    final from = text.indexOf('merci pour tout');
    c.bodyFocus.requestFocus();
    c.body.value = TextEditingValue(
      text: text,
      selection: TextSelection(
        baseOffset: from,
        extentOffset: from + 'merci pour tout'.length,
      ),
    );
    await settle(tester);
    expect(key('composer-ai-action-reformulate'), findsOneWidget);
    expect(key('composer-ai-action-shorten'), findsOneWidget);
    expect(key('composer-ai-action-fix'), findsOneWidget);
    expect(key('composer-ai-action-translate'), findsOneWidget);
    await tester.tap(key('composer-ai-action-reformulate'));
    await settle(tester);
    await settle(tester);
    expect(c.body.text, 'Salut Marie, un grand merci.\n\nCordialement');
    expect(engine.chatStarts.single['mode'], {
      'kind': 'composer',
      'selection': true,
    });
    final message =
        ((engine.chatStarts.single['request'] as Map)['messages'] as List)
                .single['text']
            as String;
    expect(message, contains('merci pour tout'));
    expect(key('composer-ai-undo'), findsOneWidget);
    await close(tester, store);
  });

  testWidgets('the_bar_does_not_overflow_on_a_narrow_pane', (tester) async {
    final (store, _) = await withComposer(tester);
    final reports = <String>[];
    final previous = FlutterError.onError;
    FlutterError.onError = (details) => reports.add(details.toString());
    tester.view.physicalSize = const Size(420, 900);
    await settle(tester);
    await tester.tap(key('composer-ai-open'));
    await settle(tester);
    expect(key('composer-ai-input'), findsOneWidget);
    FlutterError.onError = previous;
    final own = reports.where(
      (r) => r.contains('composer_ai_bar') || r.contains('composer.dart'),
    );
    expect(own, isEmpty, reason: own.join('\n'));
    await close(tester, store);
  });

  testWidgets('the_bar_lists_what_the_ai_changed_with_real_addresses', (
    tester,
  ) async {
    final (store, _) = await withComposer(
      tester,
      scripts: [
        turn([
          proposal({
            'subject': 'Fin de stage',
            'to': [
              {'name': 'Support Banque', 'address': 'evil@x.com'},
            ],
            'body': 'Texte',
          }),
        ]),
      ],
    );
    await ask(tester, 'écris');
    final changes = key('composer-ai-changes');
    expect(changes, findsOneWidget);
    expect(tester.widget<Text>(changes).data, contains('evil@x.com'));
    await close(tester, store);
  });

  testWidgets('the_undo_button_warns_when_the_user_typed_after_the_ai', (
    tester,
  ) async {
    final (store, _) = await withComposer(
      tester,
      scripts: [
        turn([
          proposal({'body': 'Texte de l’IA'}),
        ]),
      ],
    );
    await ask(tester, 'écris');
    expect(find.text('ANNULER LA MODIFICATION DE L’IA'), findsOneWidget);
    final c = store.c!;
    c.body.value = c.body.value.copyWith(text: '${c.body.text} ma phrase');
    await settle(tester);
    expect(find.textContaining('EFFACE AUSSI'), findsOneWidget);
    await close(tester, store);
  });
}
