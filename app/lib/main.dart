import 'dart:async';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:path_provider/path_provider.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:intl/date_symbol_data_local.dart';
import 'package:provider/provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'engine/engine_mail_source.dart';
import 'engine/mail_source.dart';
import 'engine/rust_engine_client.dart';
import 'i18n/i18n.dart';
import 'state/folder_map.dart';
import 'state/mail_notifier.dart';
import 'state/store.dart';
import 'theme/app_theme.dart';
import 'ui/app_root.dart';

const _forceW = int.fromEnvironment('KM_W');
const _forceH = int.fromEnvironment('KM_H', defaultValue: 800);

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  PaintingBinding.instance.imageCache
    ..maximumSize = 300
    ..maximumSizeBytes = 40 << 20;
  GoogleFonts.config.allowRuntimeFetching =
      false; // polices embarquées dans assets/google_fonts
  await Fonts.preload();
  for (final l in AppLang.values) {
    await initializeDateFormatting(l.dateLocale);
  }
  final prefs = await SharedPreferences.getInstance();
  final source = await openEngineSource(prefs);
  final store = MailStore(prefs, watchWallpaper: true, source: source);
  unawaited(store.useNotifications(PluginNotifications()));
  AppLifecycleListener(
    onStateChange: (state) =>
        store.appFocused = state == AppLifecycleState.resumed,
  );
  runApp(ChangeNotifierProvider.value(value: store, child: const MiraApp()));
}

/// Ouvre le moteur Rust ; `MIRA_DEMO=1` ou une erreur d'ouverture donne les données de démonstration.
Future<MailSource?> openEngineSource(SharedPreferences prefs) async {
  if (Platform.environment['MIRA_DEMO'] == '1') return null;
  try {
    final android = Platform.isAndroid;
    final client = await RustEngineClient.open(
      home: android ? (await getApplicationSupportDirectory()).path : null,
      secretsKey: android ? await androidSecretsKey() : null,
    );
    return await EngineMailSource.load(
      client,
      folderMap: decodeFolderMap(prefs.getString('folderMap')),
    );
  } catch (e) {
    stderr.writeln(
      'MIRA : moteur indisponible, données de démonstration ($e)',
    ); // i18n:ignore
    return null;
  }
}

/// Clé du trousseau chiffré du moteur, gardée par le Keystore Android (voir MainActivity.kt).
Future<Uint8List> androidSecretsKey() async {
  final key = await const MethodChannel('mira/keystore')
      .invokeMethod<Uint8List>('secretsKey');
  if (key == null || key.length != 32) {
    throw StateError('clé du trousseau indisponible'); // i18n:ignore
  }
  return key;
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
      title: 'MIRA',
      debugShowCheckedModeBanner: false,
      theme: buildTheme(dark, accent: accent, pal: wall),
      themeAnimationDuration: Duration.zero,
      locale: lang.locale,
      supportedLocales: [for (final l in AppLang.values) l.locale],
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
      home: const AppRoot(),
    );
  }
}
