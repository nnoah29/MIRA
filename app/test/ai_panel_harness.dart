import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:intl/date_symbol_data_local.dart';
import 'package:mira/engine/ai_admin.dart';
import 'package:mira/main.dart';
import 'package:mira/state/store.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:provider/provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'fake_ai_engine.dart';

/// Moteur factice avec une connexion OpenAI par défaut, clé enregistrée et consentement donné.
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
