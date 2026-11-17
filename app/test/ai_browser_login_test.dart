import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/ai_admin.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/state/ai_settings_state.dart';
import 'package:mira/ui/ai_browser_login.dart';

import 'ai_settings_harness.dart';
import 'fake_ai_engine.dart';

void main() {
  test('admin_start_key_login_returns_flow_and_url_without_key', () async {
    final engine = FakeAiEngine();
    final admin = AiAdmin(engine);
    final made = await admin.add(provider: 'openrouter', label: 'OR');
    final started = await admin.startKeyLogin(made.id);
    expect(started.url.host, 'openrouter.ai');
    expect(engine.requestsOf('start_ai_key_login').single, {
      'op': 'start_ai_key_login',
      'connection': made.id,
    });
    final done = await admin.completeKeyLogin(started.flow);
    expect(done.hasKey, isTrue);
  });

  test('provider_info_reads_key_login_presence', () {
    expect(
      AiProviderInfo.fromJson(provider('a', 'A', login: true)).keyLogin,
      isTrue,
    );
    expect(AiProviderInfo.fromJson(provider('b', 'B')).keyLogin, isFalse);
  });

  test('login_by_browser_opens_url_then_stores_key', () async {
    final engine = FakeAiEngine()
      ..connections.add(connection('c1', provider: 'openrouter'));
    final state = AiSettingsState(AiAdmin(engine));
    Uri? opened;
    await state.loginByBrowser('c1', (u) async {
      opened = u;
      return true;
    });
    expect(opened?.host, 'openrouter.ai');
    expect(state.connections.single.hasKey, isTrue);
    expect(state.logins, isEmpty);
    expect(state.loginLinks, isEmpty);
    expect(state.error, isNull);
  });

  test('login_keeps_link_when_browser_does_not_open', () async {
    final engine = FakeAiEngine()
      ..connections.add(connection('c1', provider: 'openrouter'));
    engine.loginGate = Completer<void>();
    final state = AiSettingsState(AiAdmin(engine));
    final running = state.loginByBrowser('c1', (_) async => false);
    await Future<void>.delayed(Duration.zero);
    await Future<void>.delayed(Duration.zero);
    expect(state.loginLinks['c1']?.host, 'openrouter.ai');
    engine.loginGate!.complete();
    await running;
    expect(state.loginLinks, isEmpty);
  });

  test('cancel_login_ends_quietly_and_keeps_no_key', () async {
    final engine = FakeAiEngine()
      ..connections.add(connection('c1', provider: 'openrouter'));
    engine.loginGate = Completer<void>();
    final state = AiSettingsState(AiAdmin(engine));
    final running = state.loginByBrowser('c1', (_) async => true);
    await Future<void>.delayed(Duration.zero);
    await Future<void>.delayed(Duration.zero);
    await state.cancelLogin('c1');
    await running;
    expect(state.error, isNull);
    expect(state.logins, isEmpty);
    expect(state.connections.single.hasKey, isFalse);
  });

  test('login_failure_shows_error_and_clears_waiting', () async {
    final engine = FakeAiEngine()
      ..connections.add(connection('c1', provider: 'openrouter'));
    engine.failOpName = 'start_ai_key_login';
    engine.failOp = const EngineException(
      'authentication.oauth_denied',
      'refusé',
    );
    final state = AiSettingsState(AiAdmin(engine));
    await state.loginByBrowser('c1', (_) async => true);
    expect(state.error, isNotNull);
    expect(state.logins, isEmpty);
  });

  testWidgets('openrouter_card_offers_browser_login_and_keeps_key_field', (
    tester,
  ) async {
    await pumpSettings(
      tester,
      connections: [connection('c1', provider: 'openrouter')],
    );
    final real = openLoginPage;
    addTearDown(() => openLoginPage = real);
    openLoginPage = (_) async => true;
    await tapKey(tester, 'ai-tab-openrouter');
    expect(byKey('ai-login-c1'), findsOneWidget);
    expect(byKey('ai-key-field-c1'), findsOneWidget);
    await tapKey(tester, 'ai-login-c1');
    expect(byKey('ai-key-saved-c1'), findsOneWidget);
  });

  testWidgets('providers_without_key_login_have_no_browser_button', (
    tester,
  ) async {
    await pumpSettings(tester, connections: [connection('c1')]);
    expect(byKey('ai-login-c1'), findsNothing);
  });
}
