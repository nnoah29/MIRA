import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/state/gif_playback.dart';
import 'package:mira/state/store.dart';

import 'app_test.dart' show pumpApp, teardown;

void main() {
  testWidgets(
    'Apparence propose les images animées, figées par défaut, et retient le choix',
    (tester) async {
      final s = await pumpApp(tester, size: const Size(1500, 1400));
      expect(s.gifPlayback, GifPlayback.off);
      s.showView(PaneView.settings);
      await tester.pump(const Duration(milliseconds: 300));
      expect(find.text('Images animées (GIF)'), findsOneWidget);
      await tester.ensureVisible(find.text('AU SURVOL'));
      await tester.tap(find.text('AU SURVOL'));
      await tester.pump();
      expect(s.gifPlayback, GifPlayback.hover);
      expect(s.prefs.getString('gifPlayback'), 'hover');
      await teardown(tester, s);
    },
  );
}
