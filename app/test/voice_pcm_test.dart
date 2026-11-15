import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/voice/pcm_convert.dart';

void main() {
  test('0x4000 donne 0,5 et 0x8000 donne -1', () {
    final d = Pcm16Decoder();
    expect(d.add(Uint8List.fromList([0x00, 0x40])), [0.5]);
    expect(d.add(Uint8List.fromList([0x00, 0x80])), [-1.0]);
  });

  test('un octet impair est gardé pour le paquet suivant', () {
    final d = Pcm16Decoder();
    final first = d.add(Uint8List.fromList([0x00, 0x40, 0x00]));
    final second = d.add(Uint8List.fromList([0x40]));
    expect(first, [0.5]);
    expect(second, [0.5]);
  });

  test('un paquet vide ne donne rien', () {
    expect(Pcm16Decoder().add(Uint8List(0)), isEmpty);
  });

  test(
    'aucun échantillon perdu ni décalé sur des paquets de tailles variées',
    () {
      final bytes = Uint8List(2000);
      for (var i = 0; i < 1000; i++) {
        final v = i * 30 - 15000;
        bytes[2 * i] = v & 0xff;
        bytes[2 * i + 1] = (v >> 8) & 0xff;
      }
      final d = Pcm16Decoder();
      final out = <double>[];
      var at = 0;
      for (final n in [1, 3, 7, 100, 333, 1556]) {
        out.addAll(d.add(bytes.sublist(at, at + n)));
        at += n;
      }
      expect(at, 2000);
      expect(out.length, 1000);
      expect(out[999], closeTo((999 * 30 - 15000) / 32768, 1e-9));
    },
  );
}
