import 'package:flutter/material.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:intl/date_symbol_data_local.dart';
import 'package:provider/provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'i18n/i18n.dart';
import 'state/store.dart';
import 'theme/app_theme.dart';
import 'ui/shell.dart';

const _forceW = int.fromEnvironment('KM_W');
const _forceH = int.fromEnvironment('KM_H', defaultValue: 800);

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  GoogleFonts.config.allowRuntimeFetching =
      false; // polices embarquées dans assets/google_fonts
  await Fonts.preload();
  await initializeDateFormatting('fr_FR');
  await initializeDateFormatting('en_US');
  final prefs = await SharedPreferences.getInstance();
  runApp(
    ChangeNotifierProvider(
      create: (_) => MailStore(prefs, watchWallpaper: true),
      child: const MiraApp(),
    ),
  );
}

class MiraApp extends StatelessWidget {
  const MiraApp({super.key});

  @override
  Widget build(BuildContext context) {
    final dark = context.select<MailStore, bool>((s) => s.isDark);
    final wall = context.select<MailStore, Pal?>((s) => s.wallPal);
    final accent = context.select<MailStore, Color?>((s) => s.accentColor);
    final lang = context.select<MailStore, AppLang>((s) => s.lang);
    return MaterialApp(
      title: 'Ma boîte — MIRA',
      debugShowCheckedModeBanner: false,
      theme: buildTheme(dark, accent: accent, pal: wall),
      themeAnimationDuration: Duration.zero,
      locale: lang.locale,
      supportedLocales: const [Locale('fr'), Locale('en')],
      localizationsDelegates: const [
        GlobalMaterialLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
      ],
      // Aide au débogage : `--dart-define=KM_W=390 --dart-define=KM_H=844` force une taille d'écran.
      builder: (ctx, child) => _forceW > 0
          ? Align(
              alignment: Alignment.topLeft,
              child: SizedBox(
                width: _forceW.toDouble(),
                height: _forceH.toDouble(),
                child: MediaQuery(
                  data: MediaQuery.of(ctx).copyWith(
                    size: Size(_forceW.toDouble(), _forceH.toDouble()),
                  ),
                  child: child!,
                ),
              ),
            )
          : child!,
      home: const Shell(),
    );
  }
}
