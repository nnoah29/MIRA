import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/voice/model_keeper.dart';

void main() {
  test(
    'le modèle est rendu une fois la durée écoulée après la dernière dictée',
    () {
      fakeAsync((async) {
        var released = 0;
        final keeper = ModelKeeper(
          keepFor: const Duration(minutes: 3),
          release: () => released++,
        );
        keeper.idle();
        async.elapse(const Duration(minutes: 2, seconds: 59));
        expect(released, 0);
        async.elapse(const Duration(seconds: 2));
        expect(released, 1);
        async.elapse(const Duration(minutes: 30));
        expect(released, 1);
      });
    },
  );

  test('une nouvelle dictée avant l’échéance annule la libération', () {
    fakeAsync((async) {
      var released = 0;
      final keeper = ModelKeeper(
        keepFor: const Duration(minutes: 3),
        release: () => released++,
      );
      keeper.idle();
      async.elapse(const Duration(minutes: 2));
      keeper.used();
      async.elapse(const Duration(minutes: 10));
      expect(released, 0);
    });
  });

  test('l’échéance repart de zéro à chaque fin de dictée', () {
    fakeAsync((async) {
      var released = 0;
      final keeper = ModelKeeper(
        keepFor: const Duration(minutes: 3),
        release: () => released++,
      );
      keeper.idle();
      async.elapse(const Duration(minutes: 2));
      keeper.used();
      keeper.idle();
      async.elapse(const Duration(minutes: 2));
      expect(released, 0);
      async.elapse(const Duration(minutes: 2));
      expect(released, 1);
    });
  });

  test('durée nulle : le modèle est rendu tout de suite', () {
    fakeAsync((async) {
      var released = 0;
      final keeper = ModelKeeper(
        keepFor: Duration.zero,
        release: () => released++,
      );
      keeper.idle();
      expect(released, 1);
    });
  });

  test('changer la durée s’applique à la prochaine fin de dictée', () {
    fakeAsync((async) {
      var released = 0;
      final keeper = ModelKeeper(
        keepFor: const Duration(minutes: 3),
        release: () => released++,
      );
      keeper.keepFor = const Duration(minutes: 1);
      keeper.idle();
      async.elapse(const Duration(minutes: 1, seconds: 1));
      expect(released, 1);
    });
  });

  test('dispose annule l’échéance sans rien rendre', () {
    fakeAsync((async) {
      var released = 0;
      final keeper = ModelKeeper(
        keepFor: const Duration(minutes: 3),
        release: () => released++,
      );
      keeper.idle();
      keeper.dispose();
      async.elapse(const Duration(minutes: 10));
      expect(released, 0);
    });
  });
}
