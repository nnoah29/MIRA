import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/ai_admin.dart';
import 'package:mira/engine/engine_mail_source.dart';
import 'package:mira/engine/remote/resident.dart';
import 'package:mira/state/store.dart';
import 'package:mira/ui/ai_browser_login.dart';
import 'package:mira/ui/onboarding/step_notifications.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'ai_settings_harness.dart' show connection, provider;
import 'engine_source_test.dart' show FakeEngine;
import 'fake_ai_engine.dart';
import 'onboarding_flow_test.dart' show pumpFlow, tapKey;

/// Contrôleur du mode résident absent : la page Notifications ne lance aucun processus.
class NoResident extends ResidentController {
  @override
  Future<bool> isAvailable() async => false;
}

Future<(MailStore, FakeAiEngine)> aiStore(
  WidgetTester tester, {
  bool withLogin = true,
  List<Map<String, dynamic>> connections = const [],
}) async {
  final real = residentForOnboarding;
  addTearDown(() => residentForOnboarding = real);
  residentForOnboarding = NoResident.new;
  SharedPreferences.setMockInitialValues({});
  final prefs = await tester.runAsync(SharedPreferences.getInstance);
  final fake = FakeAiEngine()
    ..providers.addAll([
      provider('openai', 'OpenAI'),
      provider('openrouter', 'OpenRouter', login: withLogin),
    ])
    ..connections.addAll(connections);
  final src = await tester.runAsync(
    () => EngineMailSource.load(FakeEngine()..noAccounts = true),
  );
  final store = MailStore(prefs!, source: src, aiAdminOverride: AiAdmin(fake));
  return (store, fake);
}

Future<void> toAiPage(WidgetTester tester) async {
  for (var i = 0; i < 4; i++) {
    await tapKey(tester, 'onb-next');
  }
}

void useLauncher(bool opens, List<Uri> seen) {
  final real = openLoginPage;
  addTearDown(() => openLoginPage = real);
  openLoginPage = (u) async {
    seen.add(u);
    return opens;
  };
}

void main() {
  testWidgets('ai_page_explains_privacy_and_offers_openrouter', (tester) async {
    final (store, _) = await aiStore(tester);
    await pumpFlow(tester, store: store);
    await toAiPage(tester);
    expect(find.byKey(const ValueKey('onb-ai-login')), findsOneWidget);
    expect(
      find.textContaining('Ta clé reste sur cet appareil'),
      findsOneWidget,
    );
  });

  testWidgets('ai_login_creates_connection_and_starts_browser_flow', (
    tester,
  ) async {
    final (store, fake) = await aiStore(tester);
    final seen = <Uri>[];
    useLauncher(true, seen);
    await pumpFlow(tester, store: store);
    await toAiPage(tester);
    await tapKey(tester, 'onb-ai-login');
    expect(fake.requestsOf('add_ai_connection'), hasLength(1));
    expect(fake.requestsOf('start_ai_key_login'), hasLength(1));
    expect(seen.single.host, 'openrouter.ai');
    expect(find.byKey(const ValueKey('onb-ai-login')), findsNothing);
  });

  testWidgets('ai_existing_connection_is_reused', (tester) async {
    final (store, fake) = await aiStore(
      tester,
      connections: [connection('c1', provider: 'openrouter')],
    );
    useLauncher(true, []);
    await pumpFlow(tester, store: store);
    await toAiPage(tester);
    await tapKey(tester, 'onb-ai-login');
    expect(fake.requestsOf('add_ai_connection'), isEmpty);
    expect(fake.requestsOf('start_ai_key_login').single['connection'], 'c1');
  });

  testWidgets('ai_link_remains_when_browser_does_not_open', (tester) async {
    final (store, fake) = await aiStore(tester);
    fake.loginGate = Completer<void>();
    useLauncher(false, []);
    await pumpFlow(tester, store: store);
    await toAiPage(tester);
    await tapKey(tester, 'onb-ai-login');
    expect(find.byKey(const ValueKey('onb-ai-link')), findsOneWidget);
    fake.loginGate!.complete();
    await tester.pump(const Duration(seconds: 1));
  });

  testWidgets('ai_later_moves_on_without_connection', (tester) async {
    final (store, fake) = await aiStore(tester);
    await pumpFlow(tester, store: store);
    await toAiPage(tester);
    await tapKey(tester, 'onb-ai-later');
    expect(fake.requestsOf('add_ai_connection'), isEmpty);
    expect(find.byKey(const ValueKey('onb-ai-login')), findsNothing);
  });

  testWidgets('ai_without_key_login_provider_points_to_settings', (
    tester,
  ) async {
    final (store, _) = await aiStore(tester, withLogin: false);
    await pumpFlow(tester, store: store);
    await toAiPage(tester);
    expect(find.byKey(const ValueKey('onb-ai-login')), findsNothing);
    expect(find.textContaining('Réglages › IA'), findsWidgets);
  });

  testWidgets('ai_cancel_stops_waiting_quietly', (tester) async {
    final (store, fake) = await aiStore(tester);
    fake.loginGate = Completer<void>();
    useLauncher(true, []);
    await pumpFlow(tester, store: store);
    await toAiPage(tester);
    await tapKey(tester, 'onb-ai-login');
    expect(find.byKey(const ValueKey('onb-ai-cancel')), findsOneWidget);
    await tapKey(tester, 'onb-ai-cancel');
    expect(find.byKey(const ValueKey('onb-ai-login')), findsOneWidget);
    expect(find.byKey(const ValueKey('onb-ai-error')), findsNothing);
  });
}
