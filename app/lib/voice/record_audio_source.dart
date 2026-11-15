//! Micro via le paquet `record` : PCM 16 bits mono 16 kHz converti en flottants pour la reconnaissance.

import 'dart:typed_data';

import 'package:record/record.dart';

import 'dictation_types.dart';
import 'pcm_convert.dart';

/// Source audio de toutes les plateformes ; sous Linux, `record` s'appuie sur `parecord`.
class RecordAudioSource implements AudioSource {
  AudioRecorder? _recorder;

  @override
  Future<bool> requestPermission() async {
    final probe = AudioRecorder();
    try {
      return await probe.hasPermission();
    } finally {
      await probe.dispose();
    }
  }

  @override
  Future<Stream<Float32List>> start() async {
    await stop();
    final recorder = _recorder = AudioRecorder();
    final decoder = Pcm16Decoder();
    final bytes = await recorder.startStream(
      const RecordConfig(
        encoder: AudioEncoder.pcm16bits,
        sampleRate: 16000,
        numChannels: 1,
      ),
    );
    return bytes.map(decoder.add);
  }

  @override
  Future<void> stop() async {
    final recorder = _recorder;
    _recorder = null;
    if (recorder == null) return;
    try {
      await recorder.stop();
    } finally {
      await recorder.dispose();
    }
  }
}
