//! Conversion du PCM 16 bits little-endian reçu du micro en échantillons flottants dans [-1, 1].

import 'dart:typed_data';

/// Décodeur de flux PCM 16 bits qui garde l'octet orphelin d'un paquet pour le suivant.
class Pcm16Decoder {
  int? _carry;

  /// Convertit un paquet d'octets en échantillons ; un octet impair final attend le paquet suivant.
  Float32List add(Uint8List chunk) {
    final carry = _carry;
    final data = carry == null
        ? chunk
        : (Uint8List(chunk.length + 1)
            ..[0] = carry
            ..setRange(1, chunk.length + 1, chunk));
    final count = data.length ~/ 2;
    _carry = data.length.isOdd ? data.last : null;
    final out = Float32List(count);
    final view = ByteData.sublistView(data);
    for (var i = 0; i < count; i++) {
      out[i] = view.getInt16(i * 2, Endian.little) / 32768.0;
    }
    return out;
  }
}
