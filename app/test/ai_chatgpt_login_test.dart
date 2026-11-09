import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/ai_models.dart';
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
  await tapKey(tester, 'ai-tab-chatgpt');
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
}
