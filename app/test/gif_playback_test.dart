import 'package:flutter_test/flutter_test.dart';
import 'package:mira/state/gif_playback.dart';
import 'package:mira/state/store.dart';
import 'package:shared_preferences/shared_preferences.dart';

Future<MailStore> _store(Map<String, Object> prefs) async {
  SharedPreferences.setMockInitialValues(prefs);
  return MailStore(await SharedPreferences.getInstance());
}

void main() {
  test('les images animées sont figées par défaut', () async {
    final s = await _store({});
    expect(s.gifPlayback, GifPlayback.off);
  });

  test('une valeur enregistrée inconnue donne le réglage par défaut', () {
    expect(GifPlayback.parse(null), GifPlayback.off);
    expect(GifPlayback.parse('n\'importe quoi'), GifPlayback.off);
  });

  test('les trois réglages ont un code stable', () {
    expect(GifPlayback.off.code, 'off');
    expect(GifPlayback.hover.code, 'hover');
    expect(GifPlayback.always.code, 'always');
    for (final m in GifPlayback.values) {
      expect(GifPlayback.parse(m.code), m);
    }
  });

  test(
    'le réglage change, prévient l\'interface et survit au redémarrage',
    () async {
      SharedPreferences.setMockInitialValues({});
      final prefs = await SharedPreferences.getInstance();
      final s = MailStore(prefs);
      var notified = 0;
      s.addListener(() => notified++);
      s.setSetting('gifPlayback', 'hover');
      expect(s.gifPlayback, GifPlayback.hover);
      expect(notified, greaterThan(0));
      expect(MailStore(prefs).gifPlayback, GifPlayback.hover);
      s.setSetting('gifPlayback', 'always');
      expect(MailStore(prefs).gifPlayback, GifPlayback.always);
    },
  );
}
