import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/ai_admin.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/state/ai_settings_state.dart';
import 'package:mira/ui/ai_browser_login.dart';
import 'package:mira/ui/widgets.dart';

import 'ai_settings_harness.dart';
import 'fake_ai_engine.dart';

FakeAiEngine withChatgpt() =>
    FakeAiEngine()
      ..providers.add(provider('chatgpt', 'ChatGPT', session: true));

Future<FakeAiEngine> pumpChatgpt(
  WidgetTester tester, {
  bool hasKey = false,
}) async {
  final engine = await pumpSettings(
    tester,
    using: withChatgpt(),
    connections: [connection('c1', provider: 'chatgpt', hasKey: hasKey)],
  );
  final real = openLoginPage;
  addTearDown(() => openLoginPage = real);
  openLoginPage = (_) async => true;
  await tapKey(tester, 'ai-tab-openai');
  return engine;
}

/// Libellé du bouton qui porte cette clé.
String labelOf(WidgetTester tester, String key) =>
    tester.widget<Btn>(byKey(key)).label;

void main() {
  test('provider_info_reads_session_login_presence', () {
    final info = AiProviderInfo.fromJson(provider('c', 'C', session: true));
    expect(info.sessionLogin, isTrue);
    expect(info.keyLogin, isFalse);
    expect(info.browserLogin, isTrue);
    final plain = AiProviderInfo.fromJson(provider('p', 'P'));
    expect(plain.browserLogin, isFalse);
  });

  testWidgets('chatgpt_button_requires_warning_first', (tester) async {
    final engine = await pumpChatgpt(tester);
    expect(labelOf(tester, 'ai-login-c1'), 'Continue with ChatGPT');
    await tapKey(tester, 'ai-login-c1');
    expect(byKey('ai-session-warning'), findsOneWidget);
    expect(engine.requestsOf('start_ai_key_login'), isEmpty);
    await tapKey(tester, 'ai-session-warning-cancel');
    expect(byKey('ai-session-warning'), findsNothing);
    expect(engine.requestsOf('start_ai_key_login'), isEmpty);
  });

  testWidgets('accepting_warning_starts_login', (tester) async {
    final engine = await pumpChatgpt(tester);
    await tapKey(tester, 'ai-login-c1');
    await tapKey(tester, 'ai-session-warning-accept');
    expect(engine.requestsOf('start_ai_key_login'), hasLength(1));
    expect(byKey('ai-key-saved-c1'), findsOneWidget);
  });

  testWidgets('session_provider_has_no_key_field', (tester) async {
    await pumpChatgpt(tester);
    expect(byKey('ai-key-field-c1'), findsNothing);
    expect(byKey('ai-key-save-c1'), findsNothing);
  });

  testWidgets('reconnect_button_when_connected_skips_warning', (tester) async {
    final engine = await pumpChatgpt(tester, hasKey: true);
    expect(labelOf(tester, 'ai-login-c1'), 'Se reconnecter avec ChatGPT');
    await tapKey(tester, 'ai-login-c1');
    expect(byKey('ai-session-warning'), findsNothing);
    expect(engine.requestsOf('start_ai_key_login'), hasLength(1));
  });

  testWidgets('chatgpt_shares_the_openai_tab', (tester) async {
    await pumpSettings(tester, using: withChatgpt());
    expect(byKey('ai-tab-chatgpt'), findsNothing);
    await tapKey(tester, 'ai-tab-openai');
    expect(byKey('ai-section-openai'), findsOneWidget);
    expect(byKey('ai-section-chatgpt'), findsOneWidget);
  });

  testWidgets('default_chatgpt_connection_opens_the_openai_tab', (
    tester,
  ) async {
    await pumpSettings(
      tester,
      using: withChatgpt(),
      connections: [connection('c1', provider: 'chatgpt', isDefault: true)],
    );
    expect(byKey('ai-panel-openai'), findsOneWidget);
    expect(byKey('ai-card-key-c1'), findsOneWidget);
    expect(byKey('ai-tab-default-openai'), findsOneWidget);
  });

  test('login_failure_shows_the_engine_detail', () async {
    final engine = withChatgpt()
      ..connections.add(connection('c1', provider: 'chatgpt'));
    engine.failOpName = 'complete_ai_key_login';
    engine.failOp = const EngineException(
      'authentication.rejected',
      'échange refusé par le service (HTTP 400, invalid_grant)',
    );
    final state = AiSettingsState(AiAdmin(engine));
    await state.loginByBrowser('c1', (_) async => true);
    expect(state.error, contains('HTTP 400, invalid_grant'));
  });
}
