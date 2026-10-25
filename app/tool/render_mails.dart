// Outil de développement : rend en PNG chaque .html de $MAILS_DIR avec le moteur de widgets.
// Usage : MAILS_DIR=dossier flutter test tool/render_mails.dart
import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:mira/data/models.dart';
import 'package:mira/engine/mail_layout.dart';
import 'package:mira/state/store.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/html_body.dart';
import 'package:provider/provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  testWidgets('rend les courriels du dossier MAILS_DIR en PNG', (tester) async {
    final dir = Directory(Platform.environment['MAILS_DIR']!);
    GoogleFonts.config.allowRuntimeFetching = false;
    await tester.runAsync(Fonts.preload);
    // ignore: invalid_use_of_visible_for_testing_member
    SharedPreferences.setMockInitialValues({});
    final store = MailStore(await SharedPreferences.getInstance());
    for (final f in dir.listSync().whereType<File>().where(
      (f) => f.path.endsWith('.html'),
    )) {
      final msg =
          Msg(from: const Person('L', 'l@x.org'), date: 0, body: const [])
            ..html = prepareMailHtml(f.readAsStringSync())
            ..blocked = const []
            ..inline = {};
      final key = GlobalKey();
      await tester.binding.setSurfaceSize(const Size(760, 1100));
      await tester.pumpWidget(
        ChangeNotifierProvider.value(
          value: store,
          child: MaterialApp(
            theme: buildTheme(true),
            home: Scaffold(
              body: SingleChildScrollView(
                child: RepaintBoundary(
                  key: key,
                  child: Container(
                    color: const Color(0xFF151515),
                    padding: const EdgeInsets.all(16),
                    child: MailHtml(
                      msg: msg,
                      base: const TextStyle(
                        fontSize: 15,
                        color: Colors.white70,
                      ),
                    ),
                  ),
                ),
              ),
            ),
          ),
        ),
      );
      await tester.pump();
      await tester.runAsync(
        () => Future<void>.delayed(const Duration(milliseconds: 300)),
      );
      await tester.pump(const Duration(milliseconds: 700));
      await tester.runAsync(() async {
        final b =
            key.currentContext!.findRenderObject()! as RenderRepaintBoundary;
        final img = await b.toImage(pixelRatio: 1);
        final png = await img.toByteData(format: ui.ImageByteFormat.png);
        File(f.path.replaceAll('.html', '.png'))
            .writeAsBytesSync(png!.buffer.asUint8List());
      });
    }
  });
}
