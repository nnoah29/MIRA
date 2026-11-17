import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/state/memory_trimmer.dart';
import 'package:mira/state/store.dart';
import 'package:shared_preferences/shared_preferences.dart';

class CountingTrimmer implements MemoryTrimmer {
  int calls = 0;

  @override
  void trim() => calls++;
}

Future<(MailStore, CountingTrimmer)> _store() async {
  SharedPreferences.setMockInitialValues({'hibernate': '5'});
  final trimmer = CountingTrimmer();
  final store = MailStore(
    await SharedPreferences.getInstance(),
    trimmer: trimmer,
  );
  return (store, trimmer);
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test(
    'l’hibernation rend la mémoire libre au système, puis une seconde fois',
    () async {
      final (s, trimmer) = await _store();
      fakeAsync((async) {
        s.appFocused = false;
        async.elapse(const Duration(minutes: 4));
        expect(trimmer.calls, 0);
        async.elapse(const Duration(minutes: 1, seconds: 10));
        expect(s.hibernating, isTrue);
        expect(trimmer.calls, 1);
        async.elapse(const Duration(seconds: 30));
        expect(trimmer.calls, 2);
        async.elapse(const Duration(minutes: 10));
        expect(trimmer.calls, 2);
      });
    },
  );

  test('le focus revient avant la seconde passe : elle est annulée', () async {
    final (s, trimmer) = await _store();
    fakeAsync((async) {
      s.appFocused = false;
      async.elapse(const Duration(minutes: 5, seconds: 10));
      expect(trimmer.calls, 1);
      s.appFocused = true;
      async.elapse(const Duration(minutes: 1));
      expect(trimmer.calls, 1);
    });
  });

  test('sans hibernation (focus gardé), aucune libération', () async {
    final (s, trimmer) = await _store();
    fakeAsync((async) {
      s.appFocused = false;
      async.elapse(const Duration(minutes: 3));
      s.appFocused = true;
      async.elapse(const Duration(minutes: 20));
      expect(trimmer.calls, 0);
    });
  });
}
