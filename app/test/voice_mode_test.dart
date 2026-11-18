import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/voice/dictation_controller.dart';
import 'package:mira/voice/dictation_types.dart';

import 'voice_fakes.dart';

DictationController make() => DictationController(
  audio: FakeAudioSource(),
  recognizer: FakeRecognizer(),
  models: FakeModels(),
);

void main() {
  test('par défaut la dictée est continue', () async {
    final c = make();
    await c.start(TextEditingController());
    expect(c.mode, DictationMode.continuous);
  });

  test(
    'phrase par phrase : le mode est mémorisé le temps de la dictée',
    () async {
      final c = make();
      await c.start(TextEditingController(), mode: DictationMode.pushToTalk);
      expect(c.mode, DictationMode.pushToTalk);
      await c.stop();
      expect(c.mode, DictationMode.continuous);
    },
  );

  test('une dictée qui échoue ne garde pas le mode', () async {
    final c = DictationController(
      audio: FakeAudioSource(),
      recognizer: FakeRecognizer(),
      models: FakeModels({}),
    );
    await c.start(TextEditingController(), mode: DictationMode.pushToTalk);
    expect(c.state, DictationState.idle);
    expect(c.mode, DictationMode.continuous);
  });
}
