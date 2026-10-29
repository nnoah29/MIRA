import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/ai_consent_dialog.dart';
import 'package:mira/ui/step_animator.dart';

import 'ai_panel_harness.dart';
import 'fake_ai_engine.dart';

const consentError = {
  'type': 'error',
  'error': {'code': 'incompatible_state.ai_consent_required', 'message': 'x'},
};

FakeAiEngine needingConsent() {
  final engine = withDefaultConnection();
  engine.connections.single['consent_at'] = null;
  return engine;
}

void main() {
  setUpAll(() => StepAnimator.frameCap = null);

  testWidgets('dialog_names_the_provider', (tester) async {
    bool? answer;
    await tester.pumpWidget(
      MaterialApp(
        theme: buildTheme(false),
        home: Builder(
          builder: (context) => TextButton(
            onPressed: () async =>
                answer = await showAiConsentDialog(context, 'Mistral AI'),
            child: const Text('go'),
          ),
        ),
      ),
    );
    await tester.tap(find.text('go'));
    await tester.pumpAndSettle();
    expect(find.textContaining('Mistral AI'), findsWidgets);
    await tester.tap(find.byKey(const ValueKey('ai-consent-accept')));
    await tester.pumpAndSettle();
    expect(answer, isTrue);
  });

  testWidgets('refusal_returns_false_and_sends_nothing', (tester) async {
    final engine = needingConsent()..scripts.add([consentError]);
    final store = await pumpPanel(tester, engine);
    await tester.tap(find.text('RÉSUME CE MAIL'));
    await tester.pump(const Duration(milliseconds: 200));
    await tester.pump(const Duration(milliseconds: 200));
    expect(find.textContaining('OpenAI'), findsWidgets);
    await tester.tap(find.byKey(const ValueKey('ai-consent-refuse')));
    await tester.pump(const Duration(milliseconds: 300));
    expect(engine.requestsOf('acknowledge_ai_consent'), isEmpty);
    expect(engine.chatStarts, hasLength(1));
    expect(store.chat.last.text, contains('rien n’a été transmis'));
    await close(tester, store);
  });

  testWidgets('acceptance_calls_acknowledge_then_resends', (tester) async {
    final engine = needingConsent()
      ..scripts.addAll([
        [consentError],
        [
          {'type': 'text', 'text': 'Voici.'},
          {'type': 'done', 'reason': 'end_turn'},
        ],
      ]);
    final store = await pumpPanel(tester, engine);
    await tester.tap(find.text('RÉSUME CE MAIL'));
    await tester.pump(const Duration(milliseconds: 200));
    await tester.pump(const Duration(milliseconds: 200));
    await tester.tap(find.byKey(const ValueKey('ai-consent-accept')));
    await tester.pump(const Duration(milliseconds: 300));
    await tester.pump(const Duration(milliseconds: 300));
    expect(engine.requestsOf('acknowledge_ai_consent'), hasLength(1));
    expect(engine.chatStarts, hasLength(2));
    expect(store.chat.last.text, 'Voici.');
    await close(tester, store);
  });

  testWidgets('dialog_shows_the_destination_host_for_a_free_address', (
    tester,
  ) async {
    await tester.pumpWidget(
      MaterialApp(
        theme: buildTheme(false),
        home: Builder(
          builder: (context) => TextButton(
            onPressed: () => showAiConsentDialog(
              context,
              'Compatible OpenAI',
              host: 'ia.example.org',
            ),
            child: const Text('go'),
          ),
        ),
      ),
    );
    await tester.tap(find.text('go'));
    await tester.pumpAndSettle();
    expect(find.textContaining('ia.example.org'), findsOneWidget);
  });

  testWidgets('panel_passes_the_host_of_a_generic_connection', (tester) async {
    final engine = needingConsent()..scripts.add([consentError]);
    engine.connections.single
      ..['provider'] = 'generic'
      ..['base_url'] = 'https://ia.example.org/v1';
    engine.providers.add({
      'id': 'generic',
      'display_name': 'Compatible OpenAI',
      'base_url': 'https://x/v1',
      'base_url_editable': true,
    });
    final store = await pumpPanel(tester, engine);
    await tester.tap(find.text('RÉSUME CE MAIL'));
    await tester.pump(const Duration(milliseconds: 200));
    await tester.pump(const Duration(milliseconds: 200));
    expect(find.byKey(const ValueKey('ai-consent-host')), findsOneWidget);
    expect(find.textContaining('ia.example.org'), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('ai-consent-refuse')));
    await tester.pump(const Duration(milliseconds: 300));
    await close(tester, store);
  });
}
