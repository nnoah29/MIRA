import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/platform_info.dart';

void main() {
  test('le système détecté est celui de la machine de test', () {
    expect(runsOnLinux, Platform.isLinux);
    expect(runsOnAndroid, Platform.isAndroid);
  });
}
