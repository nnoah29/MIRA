import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:mira/app_info.dart';
import 'package:mira/i18n/i18n.dart';
import 'package:mira/state/store.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/about_settings.dart';
import 'package:mira/ui/onboarding/onboarding_flow.dart';
import 'package:provider/provider.dart';

import 'onboarding_test.dart' show open;

Future<MailStore> pumpFlow(
  WidgetTester tester, {
  Size? size,
  MailStore? store,
  double textScale = 1,
}) async {
  GoogleFonts.config.allowRuntimeFetching = false;
  await tester.runAsync(Fonts.preload);
  tester.view.physicalSize = size ?? const Size(1200, 900);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.resetPhysicalSize);
  final used = store ?? await tester.runAsync(open) as MailStore;
  addTearDown(() => appLang = AppLang.fr);
  await tester.pumpWidget(
    ChangeNotifierProvider.value(
      value: used,
      child: MaterialApp(
        theme: buildTheme(true),
        builder: (context, child) => MediaQuery(
          data: MediaQuery.of(context)
              .copyWith(textScaler: TextScaler.linear(textScale)),
          child: child!,
        ),
        home: const OnboardingFlow(),
      ),
    ),
  );
  await tester.pump(const Duration(milliseconds: 400));
  return used;
}

Future<void> tapKey(WidgetTester tester, String key) async {
  await tester.tap(find.byKey(ValueKey(key)));
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 400));
  await tester.pump(const Duration(milliseconds: 400));
}

void main() {
  testWidgets('flow_starts_on_welcome_with_title', (tester) async {
    await pumpFlow(tester);
    expect(find.text('TES MAILS. CHEZ TOI.'), findsOneWidget);
    expect(find.byKey(const ValueKey('onb-next')), findsOneWidget);
    expect(find.byKey(const ValueKey('onb-back')), findsNothing);
  });

  testWidgets('next_and_back_walk_the_pages', (tester) async {
    await pumpFlow(tester);
    await tapKey(tester, 'onb-next');
    expect(find.textContaining('Aucun serveur MIRA'), findsOneWidget);
    await tapKey(tester, 'onb-back');
    expect(find.text('TES MAILS. CHEZ TOI.'), findsOneWidget);
  });

  testWidgets('dots_reflect_visited_pages', (tester) async {
    await pumpFlow(tester);
    await tapKey(tester, 'onb-next');
    await tapKey(tester, 'onb-next');
    await tapKey(tester, 'onb-dot-0');
    expect(find.text('TES MAILS. CHEZ TOI.'), findsOneWidget);
    await tapKey(tester, 'onb-dot-4');
    expect(find.text('TES MAILS. CHEZ TOI.'), findsOneWidget);
  });

  testWidgets('skip_ends_and_marks_welcomed', (tester) async {
    final store = await pumpFlow(tester);
    await tapKey(tester, 'onb-skip');
    expect(store.welcomeOpen, isFalse);
    expect(store.prefs.getString('welcomed'), '1');
    expect(store.wantsAddAccount, isFalse);
  });

  testWidgets('keyboard_arrows_and_escape', (tester) async {
    final store = await pumpFlow(tester);
    await tester.sendKeyEvent(LogicalKeyboardKey.arrowRight);
    await tester.pump(const Duration(milliseconds: 400));
    expect(find.textContaining('Aucun serveur MIRA'), findsOneWidget);
    await tester.sendKeyEvent(LogicalKeyboardKey.arrowLeft);
    await tester.pump(const Duration(milliseconds: 400));
    expect(find.text('TES MAILS. CHEZ TOI.'), findsOneWidget);
    await tester.sendKeyEvent(LogicalKeyboardKey.escape);
    await tester.pump(const Duration(milliseconds: 400));
    expect(store.welcomeOpen, isFalse);
  });

  testWidgets('welcome_language_switch', (tester) async {
    final store = await pumpFlow(tester);
    await tapKey(tester, 'onb-lang-en');
    expect(store.lang, AppLang.en);
    expect(find.text('START'), findsOneWidget);
  });

  testWidgets('ready_page_finishes_without_account_form', (tester) async {
    final store = await pumpFlow(tester);
    for (var i = 0; i < 6; i++) {
      await tapKey(tester, 'onb-next');
    }
    expect(find.byKey(const ValueKey('onb-finish')), findsOneWidget);
    await tapKey(tester, 'onb-finish');
    expect(store.welcomeOpen, isFalse);
    expect(store.wantsAddAccount, isFalse);
  });

  testWidgets('privacy_link_opens_policy', (tester) async {
    await pumpFlow(tester);
    Uri? opened;
    final real = openLink;
    addTearDown(() => openLink = real);
    openLink = (u) async {
      opened = u;
      return true;
    };
    await tapKey(tester, 'onb-next');
    await tapKey(tester, 'onb-privacy-link');
    expect(opened, Uri.parse(privacyUrl));
  });

  testWidgets('enter_moves_on_and_finishes_on_the_last_page', (tester) async {
    final store = await pumpFlow(tester);
    await tester.sendKeyEvent(LogicalKeyboardKey.enter);
    await tester.pump(const Duration(milliseconds: 600));
    expect(find.textContaining('Aucun serveur MIRA'), findsOneWidget);
    for (var i = 0; i < 5; i++) {
      await tapKey(tester, 'onb-next');
    }
    expect(find.byKey(const ValueKey('onb-finish')), findsOneWidget);
    await tester.sendKeyEvent(LogicalKeyboardKey.enter);
    await tester.pump(const Duration(milliseconds: 600));
    expect(store.welcomeOpen, isFalse);
  });
}
