import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/remote/resident.dart';
import 'package:mira/i18n/i18n.dart';
import 'package:mira/ui/onboarding/step_notifications.dart';

import 'dart:async';

import 'package:mira/ui/ai_browser_login.dart';

import 'onboarding_ai_test.dart' show aiStore;
import 'onboarding_flow_test.dart' show pumpFlow, tapKey;

class AvailableResident extends ResidentController {
  @override
  Future<bool> isAvailable() async => true;

  @override
  Future<bool> isEnabled() async => false;
}

Future<void> walkAllPages(WidgetTester tester) async {
  expect(tester.takeException(), isNull);
  for (var i = 0; i < 6; i++) {
    await tapKey(tester, 'onb-next');
    expect(tester.takeException(), isNull, reason: 'page ${i + 2}');
  }
}

void main() {
  for (final lang in [AppLang.fr, AppLang.en]) {
    testWidgets('no_page_overflows_on_narrow_screen_${lang.code}', (
      tester,
    ) async {
      final real = residentForOnboarding;
      addTearDown(() => residentForOnboarding = real);
      residentForOnboarding = AvailableResident.new;
      final store = await pumpFlow(
        tester,
        size: const Size(360, 640),
        textScale: 1.3,
      );
      store.setLang(lang);
      await tester.pump(const Duration(milliseconds: 400));
      await walkAllPages(tester);
      expect(find.byKey(const ValueKey('onb-finish')), findsOneWidget);
    });
  }

  for (final lang in [AppLang.fr, AppLang.en]) {
    testWidgets('ai_page_fits_narrow_screen_${lang.code}', (tester) async {
      final (store, fake) = await aiStore(tester);
      fake.loginGate = Completer<void>();
      final real = openLoginPage;
      addTearDown(() => openLoginPage = real);
      openLoginPage = (_) async => false;
      await pumpFlow(
        tester,
        store: store,
        size: const Size(360, 640),
        textScale: 1.3,
      );
      store.setLang(lang);
      await tester.pump(const Duration(milliseconds: 400));
      for (var i = 0; i < 4; i++) {
        await tapKey(tester, 'onb-next');
      }
      expect(tester.takeException(), isNull, reason: 'bouton de connexion');
      await tapKey(tester, 'onb-ai-login');
      expect(tester.takeException(), isNull, reason: 'attente et lien');
      fake.loginGate!.complete();
      await tester.pump(const Duration(seconds: 1));
    });
  }
}
