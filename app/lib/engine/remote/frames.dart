import 'dart:convert';
import 'dart:typed_data';

/// Taille maximale d'un corps de trame (16 Mio), comme le démon.
const int maxFrameBytes = 16 * 1024 * 1024;

/// Trame que le protocole refuse : trop grande, vide ou illisible.
class ProtocolException implements Exception {
  /// `too_large`, `empty` ou `malformed`.
  final String reason;
  const ProtocolException(this.reason);

  @override
  String toString() => 'ProtocolException($reason)';
}

/// Écrit une trame : quatre octets de longueur grand-boutistes puis le JSON UTF-8.
Uint8List encodeFrame(Map<String, dynamic> frame, {int max = maxFrameBytes}) {
  final body = utf8.encode(jsonEncode(frame));
  if (body.length > max) throw const ProtocolException('too_large');
  final out = Uint8List(4 + body.length);
  ByteData.sublistView(out).setUint32(0, body.length);
  out.setRange(4, out.length, body);
  return out;
}

/// Décodeur incrémental : reçoit des morceaux d'octets quelconques et rend les trames complètes.
class FrameDecoder {
  /// Taille maximale annoncée acceptée.
  final int max;
  FrameDecoder({this.max = maxFrameBytes});

  final Uint8List _header = Uint8List(4);
  int _headerFill = 0;
  BytesBuilder? _body;
  int _remaining = 0;

  /// Ajoute un morceau ; rend les trames devenues complètes (zéro, une ou plusieurs).
  /// La taille annoncée est jugée dès l'en-tête complet, avant d'attendre le corps.
  List<Map<String, dynamic>> add(List<int> chunk) {
    final frames = <Map<String, dynamic>>[];
    var i = 0;
    while (i < chunk.length) {
      if (_body == null) {
        i += _fillHeader(chunk, i);
        continue;
      }
      final take = _remaining < chunk.length - i
          ? _remaining
          : chunk.length - i;
      _body!.add(chunk.sublist(i, i + take));
      _remaining -= take;
      i += take;
      if (_remaining == 0) {
        frames.add(_parse(_body!.takeBytes()));
        _body = null;
      }
    }
    return frames;
  }

  /// Copie des octets d'en-tête ; l'en-tête complet fixe la taille attendue après contrôle.
  int _fillHeader(List<int> chunk, int from) {
    final take = (4 - _headerFill) < chunk.length - from
        ? 4 - _headerFill
        : chunk.length - from;
    _header.setRange(_headerFill, _headerFill + take, chunk, from);
    _headerFill += take;
    if (_headerFill < 4) return take;
    _headerFill = 0;
    final length = ByteData.sublistView(_header).getUint32(0);
    if (length == 0) throw const ProtocolException('empty');
    if (length > max) throw const ProtocolException('too_large');
    _body = BytesBuilder(copy: false);
    _remaining = length;
    return take;
  }

  /// Lit le corps JSON : il doit être un objet.
  Map<String, dynamic> _parse(Uint8List body) {
    try {
      final value = jsonDecode(utf8.decode(body));
      if (value is Map<String, dynamic>) return value;
    } on FormatException {
      throw const ProtocolException('malformed');
    }
    throw const ProtocolException('malformed');
  }
}
