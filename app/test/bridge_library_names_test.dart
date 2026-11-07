import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/bridge_library_io.dart';

void main() {
  test('le nom de la bibliothèque suit la plateforme', () {
    expect(bridgeLibraryFileName('windows'), 'mira_bridge.dll');
    expect(bridgeLibraryFileName('linux'), 'libmira_bridge.so');
    expect(bridgeLibraryFileName('android'), 'libmira_bridge.so');
  });

  test('sous Windows, la DLL est cherchée à côté de l’exécutable', () {
    final found = bridgeLibraryPaths(
      os: 'windows',
      executableDir: r'C:\Mira',
      override: null,
    );
    expect(found.first, r'C:\Mira/mira_bridge.dll');
    expect(found.last, 'mira_bridge.dll');
  });

  test('la variable d’environnement passe avant tout', () {
    final found = bridgeLibraryPaths(
      os: 'linux',
      executableDir: '/opt/mira',
      override: '/x/lib.so',
    );
    expect(found.first, '/x/lib.so');
    expect(found[1], '/opt/mira/lib/libmira_bridge.so');
  });
}
