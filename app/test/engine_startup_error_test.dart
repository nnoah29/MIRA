import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/main.dart' show engineFailureText;
import 'package:mira/state/store.dart';
import 'package:shared_preferences/shared_preferences.dart';

Future<SharedPreferences> prefs() async {
  SharedPreferences.setMockInitialValues({});
  return SharedPreferences.getInstance();
}

void main() {
  test(
    'un moteur indisponible est signalé à l\'écran avec sa raison',
    () async {
      final store = MailStore(
        await prefs(),
        engineError: 'clé du trousseau indisponible',
      );
      expect(store.toasts, hasLength(1));
      expect(store.toasts.single.err, isTrue);
      expect(
        store.toasts.single.msg,
        contains('clé du trousseau indisponible'),
      );
      store.dispose();
    },
  );

  test('la démo en ligne prévient que les données sont fictives', () async {
    final store = MailStore(await prefs(), webDemo: true);
    expect(store.toasts, hasLength(1));
    expect(store.toasts.single.err, isFalse);
    expect(store.toasts.single.msg, contains('fictives'));
    store.dispose();
  });

  test('un moteur qui s\'ouvre ne laisse aucun message', () async {
    final store = MailStore(await prefs());
    expect(store.toasts, isEmpty);
    store.dispose();
  });

  test('la raison d\'un échec Android cite la cause du Keystore', () {
    final error = PlatformException(
      code: 'keystore',
      message: 'clé du trousseau indisponible',
      details: 'AEADBadTagException',
    );
    expect(
      engineFailureText(error),
      'clé du trousseau indisponible (AEADBadTagException)',
    );
  });

  test('la raison d\'un échec sans détail est le message seul', () {
    final error = PlatformException(code: 'keystore', message: 'indisponible');
    expect(engineFailureText(error), 'indisponible');
    expect(engineFailureText(StateError('boum')), contains('boum'));
  });
}
