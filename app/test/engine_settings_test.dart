import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/engine_mail_source.dart';
import 'package:mira/state/store.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'engine_source_test.dart' show FakeEngine;

/// Ouvre le magasin sur le faux moteur, ses réglages lus.
Future<(MailStore, SharedPreferences)> open(
  FakeEngine fake, [
  Map<String, Object> prefs = const {},
]) async {
  SharedPreferences.setMockInitialValues(prefs);
  final p = await SharedPreferences.getInstance();
  final store = MailStore(p, source: await EngineMailSource.load(fake));
  await store.localLabelsReady;
  await settle();
  return (store, p);
}

Future<void> settle() => Future<void>.delayed(const Duration(milliseconds: 50));

void main() {
  test('le relevé affiché est celui du moteur', () async {
    final fake = FakeEngine()
      ..engineSettingsOn = true
      ..enginePoll = 120;
    final (store, _) = await open(fake);
    expect(store.pollSecs, 120);
    expect(store.pollChoice, '120');
    store.dispose();
  });

  test(
    'choisir un relevé l’envoie au moteur, sans préférence locale',
    () async {
      final fake = FakeEngine()..engineSettingsOn = true;
      final (store, prefs) = await open(fake);
      store.setSetting('poll', '60');
      await settle();
      expect(fake.enginePoll, 60);
      expect(store.pollChoice, '60');
      expect(prefs.getString('poll'), isNull);
      store.setSetting('poll', '');
      await settle();
      expect(fake.enginePoll, isNull);
      store.dispose();
    },
  );

  test('un changement venu du moteur met l’affichage à jour', () async {
    final fake = FakeEngine()..engineSettingsOn = true;
    final (store, _) = await open(fake);
    expect(store.pollSecs, isNull);
    fake.enginePoll = 300;
    fake.feed.add({
      'kind': {
        'type': 'settings_changed',
        'keys': ['poll_secs'],
      },
    });
    await settle();
    expect(store.pollChoice, '300');
    store.dispose();
  });

  test(
    'l’ancien relevé des préférences est repris une fois par le moteur',
    () async {
      final fake = FakeEngine()..engineSettingsOn = true;
      final (store, prefs) = await open(fake, {'poll': '30'});
      expect(fake.enginePoll, 30);
      expect(store.pollChoice, '30');
      expect(prefs.getString('poll'), isNull);
      store.dispose();
    },
  );

  test(
    'un relevé déjà fixé dans le moteur n’est pas écrasé par les préférences',
    () async {
      final fake = FakeEngine()
        ..engineSettingsOn = true
        ..enginePoll = 600;
      final (store, prefs) = await open(fake, {'poll': '30'});
      expect(fake.settingWrites, isEmpty);
      expect(store.pollChoice, '600');
      expect(prefs.getString('poll'), isNull);
      store.dispose();
    },
  );

  test('moteur sans réglages : les préférences restent la source', () async {
    final fake = FakeEngine();
    final (store, prefs) = await open(fake, {'poll': '30'});
    expect(prefs.getString('poll'), '30');
    store.dispose();
  });
}
