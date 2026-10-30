import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';

(int, int) pngSize(String path) {
  final bytes = File(path).readAsBytesSync();
  final header = ByteData.sublistView(bytes, 16, 24);
  return (header.getUint32(0), header.getUint32(4));
}

void main() {
  test('le favicon est assez grand pour être net (pas le 16 px de Flutter)', () {
    final (width, height) = pngSize('web/favicon.png');
    expect(width, greaterThanOrEqualTo(32));
    expect(width, height);
  });

  test('les icônes de la PWA ont la taille annoncée par le manifeste', () {
    for (final size in [192, 512]) {
      expect(pngSize('web/icons/Icon-$size.png'), (size, size));
      expect(pngSize('web/icons/Icon-maskable-$size.png'), (size, size));
    }
  });
}
