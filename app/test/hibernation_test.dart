import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/state/store.dart';
import 'package:shared_preferences/shared_preferences.dart';

Future<MailStore> _store(Map<String, Object> prefs) async {
  SharedPreferences.setMockInitialValues(prefs);
  return MailStore(await SharedPreferences.getInstance());
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  test(
    'sans le focus au-delà du délai, l\'interface hiberne puis se réveille',
    () async {
      final s = await _store({'hibernate': '5'});
      fakeAsync((async) {
        s.appFocused = false;
        async.elapse(const Duration(minutes: 4));
        expect(s.hibernating, isFalse);
        async.elapse(const Duration(minutes: 2));
        expect(s.hibernating, isTrue);
        s.appFocused = true;
        expect(s.hibernating, isFalse);
      });
    },
  );

  test('revenir avant le délai annule l\'hibernation', () async {
    final s = await _store({'hibernate': '5'});
    fakeAsync((async) {
      s.appFocused = false;
      async.elapse(const Duration(minutes: 3));
      s.appFocused = true;
      async.elapse(const Duration(minutes: 10));
      expect(s.hibernating, isFalse);
    });
  });

  test('« Jamais » désactive l\'hibernation', () async {
    final s = await _store({'hibernate': '0'});
    fakeAsync((async) {
      s.appFocused = false;
      async.elapse(const Duration(hours: 5));
      expect(s.hibernating, isFalse);
    });
  });

  test('le délai par défaut est de 10 minutes et se règle', () async {
    final s = await _store({});
    expect(s.hibernateMinutes, 10);
    s.setSetting('hibernate', '30');
    expect(s.hibernateMinutes, 30);
  });
}
