import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:intl/date_symbol_data_local.dart';
import 'package:mira/engine/ai_admin.dart';
import 'package:mira/main.dart';
import 'package:mira/state/store.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/step_animator.dart';
import 'package:provider/provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'fake_ai_engine.dart';

FakeAiEngine withDefaultConnection() => FakeAiEngine()
  ..providers.add({
    'id': 'openai',
    'display_name': 'OpenAI',
    'base_url': 'https://api.openai.com/v1',
    'base_url_editable': false,
  })
  ..connections.add({
    'id': 'ai-1',
    'provider': 'openai',
    'label': 'Mon OpenAI',
    'base_url': null,
    'default_model': 'm1',
    'is_default': true,
    'has_key': true,
    'consent_at': 1,
    'models': null,
  });

Future<MailStore> pumpPanel(WidgetTester tester, FakeAiEngine? engine) async {
  GoogleFonts.config.allowRuntimeFetching = false;
  SharedPreferences.setMockInitialValues({});
  await initializeDateFormatting('fr_FR');
  await tester.runAsync(Fonts.preload);
  tester.view.physicalSize = const Size(1500, 950);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.resetPhysicalSize);
  final store = MailStore(
    await SharedPreferences.getInstance(),
    aiAdminOverride: engine == null ? null : AiAdmin(engine),
  );
  await tester.pumpWidget(
    ChangeNotifierProvider.value(value: store, child: const MiraApp()),
  );
  await tester.pump(const Duration(milliseconds: 100));
  await tester.tap(find.text('ASSISTANT IA').first);
  await tester.pump(const Duration(milliseconds: 300));
  await tester.pump(const Duration(milliseconds: 50));
  return store;
}

Future<void> close(WidgetTester tester, MailStore store) async {
  await tester.pumpWidget(const SizedBox());
  store.dispose();
}

void main() {
  setUpAll(() => StepAnimator.frameCap = null);

  testWidgets('demo_banner_visible_without_connection', (tester) async {
    final store = await pumpPanel(tester, null);
    expect(find.byKey(const ValueKey('demo-banner')), findsOneWidget);
    await close(tester, store);
  });

  testWidgets('demo_banner_visible_with_engine_but_no_connection', (
    tester,
  ) async {
    final store = await pumpPanel(tester, FakeAiEngine());
    expect(find.byKey(const ValueKey('demo-banner')), findsOneWidget);
    await close(tester, store);
  });

  testWidgets('banner_hidden_when_a_connection_answers', (tester) async {
    final store = await pumpPanel(tester, withDefaultConnection());
    expect(find.byKey(const ValueKey('demo-banner')), findsNothing);
    await close(tester, store);
  });

  testWidgets('stop_button_visible_while_busy', (tester) async {
    final engine = withDefaultConnection()..manual = true;
    final store = await pumpPanel(tester, engine);
    expect(find.byKey(const ValueKey('chat-send')), findsOneWidget);
    await tester.tap(find.text('RÉSUME CE MAIL'));
    await tester.pump(const Duration(milliseconds: 50));
    expect(find.byKey(const ValueKey('chat-stop')), findsOneWidget);
    expect(find.byKey(const ValueKey('chat-send')), findsNothing);
    await tester.tap(find.byKey(const ValueKey('chat-stop')));
    await tester.pump(const Duration(milliseconds: 50));
    expect(find.byKey(const ValueKey('chat-send')), findsOneWidget);
    await close(tester, store);
  });

  testWidgets('insert_in_reply_opens_composer_with_text', (tester) async {
    final engine = withDefaultConnection()
      ..scripts.add([
        {'type': 'text', 'text': 'Bonjour **Bob**, merci.'},
        {'type': 'done', 'reason': 'end_turn'},
      ]);
    final store = await pumpPanel(tester, engine);
    await tester.tap(find.text('RÉSUME CE MAIL'));
    await tester.pump(const Duration(milliseconds: 100));
    await tester.pump(const Duration(milliseconds: 300));
    expect(store.chat.last.text, 'Bonjour **Bob**, merci.');
    await tester.tap(find.byKey(const ValueKey('insert-in-reply')));
    await tester.pump(const Duration(milliseconds: 300));
    expect(store.view, PaneView.compose);
    expect(store.c!.body.text, contains('Bonjour Bob, merci.'));
    expect(store.c!.body.text, isNot(contains('**')));
    await close(tester, store);
  });
}
