import 'dart:async';
import 'dart:typed_data';

import 'package:mira/voice/dictation_types.dart';

/// Micro factice : on y pousse des échantillons à la main.
class FakeAudioSource implements AudioSource {
  bool permission = true;
  Object? startError;
  int startCount = 0;
  int stopCount = 0;
  int permissionAsked = 0;
  StreamController<Float32List> out = StreamController<Float32List>();

  @override
  Future<bool> requestPermission() async {
    permissionAsked++;
    return permission;
  }

  @override
  Future<Stream<Float32List>> start() async {
    if (startError != null) throw startError!;
    startCount++;
    out = StreamController<Float32List>();
    return out.stream;
  }

  @override
  Future<void> stop() async {
    stopCount++;
    if (!out.isClosed) unawaited(out.close());
  }
}

/// Reconnaisseur factice : émet ce qu'on lui dit, de façon synchrone.
class FakeRecognizer implements SpeechRecognizer {
  final StreamController<DictationEvent> _events =
      StreamController<DictationEvent>.broadcast(sync: true);
  DictationException? startError;
  DictationLanguage? startedWith;
  String? textOnStop;
  int startCount = 0;
  int stopCount = 0;
  final List<Float32List> fed = [];

  @override
  Stream<DictationEvent> get events => _events.stream;

  @override
  Future<void> start(DictationLanguage language) async {
    if (startError != null) throw startError!;
    startCount++;
    startedWith = language;
  }

  @override
  void feed(Float32List samples) => fed.add(samples);

  @override
  Future<void> stop() async {
    stopCount++;
    final t = textOnStop;
    if (t != null) _events.add(DictationFinal(t));
  }

  void say(DictationEvent e) => _events.add(e);
  void fail() => _events.addError(StateError('boom'));
}

/// Modèles factices : l'ensemble des langues installées.
class FakeModels implements ModelAvailability {
  FakeModels([Set<DictationLanguage>? installed])
    : installed = installed ?? {DictationLanguage.fr, DictationLanguage.en};
  final Set<DictationLanguage> installed;

  @override
  bool isInstalled(DictationLanguage language) => installed.contains(language);
}
