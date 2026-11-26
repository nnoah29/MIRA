import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/state/unsubscribe.dart';

Future<HttpServer> serveCounting(List<String> hits) async {
  final server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
  server.listen((r) {
    hits.add(r.headers.host ?? '');
    r.response.close();
  });
  return server;
}

SecurityContext serverContext() => SecurityContext()
  ..useCertificateChain('test/fixtures/tls/rebind.pem')
  ..usePrivateKey('test/fixtures/tls/rebind.key');

SecurityContext trustContext() =>
    SecurityContext(withTrustedRoots: false)
      ..setTrustedCertificates('test/fixtures/tls/rebind.pem');

void main() {
  test(
    'la connexion va à l\'adresse contrôlée, sans nouvelle résolution',
    () async {
      final hits = <String>[];
      final server = await serveCounting(hits);
      var lookups = 0;
      final u = Unsubscriber()
        ..allowAddress = ((_) => true)
        ..resolve = (host) async {
          lookups++;
          return [InternetAddress.loopbackIPv4];
        };
      final ok = await u.pinAndPost(
        Uri.parse('http://rebind.invalid:${server.port}/u'),
      );
      await server.close(force: true);
      expect(ok, isTrue);
      expect(hits, ['rebind.invalid']);
      expect(lookups, 1);
    },
  );

  test(
    'une résolution mêlant public et privé est refusée sans requête',
    () async {
      final hits = <String>[];
      final server = await serveCounting(hits);
      final u = Unsubscriber()
        ..resolve = (host) async => [
          InternetAddress('8.8.8.8'),
          InternetAddress.loopbackIPv4,
        ];
      final ok = await u.pinAndPost(
        Uri.parse('http://rebind.invalid:${server.port}/u'),
      );
      await server.close(force: true);
      expect(ok, isFalse);
      expect(hits, isEmpty);
    },
  );

  test(
    'en https, le certificat est vérifié sur le nom et non sur l\'adresse',
    () async {
      final server = await HttpServer.bindSecure(
        InternetAddress.loopbackIPv4,
        0,
        serverContext(),
      );
      final hits = <String>[];
      server.listen((r) {
        hits.add(r.headers.host ?? '');
        r.response.close();
      });
      final u = Unsubscriber()
        ..allowAddress = ((_) => true)
        ..tlsContext = trustContext()
        ..resolve = (host) async => [InternetAddress.loopbackIPv4];
      final ok = await u.pinAndPost(
        Uri.parse('https://rebind.invalid:${server.port}/u'),
      );
      final other = await u.pinAndPost(
        Uri.parse('https://autre.invalid:${server.port}/u'),
      );
      await server.close(force: true);
      expect(ok, isTrue);
      expect(other, isFalse);
      expect(hits, ['rebind.invalid']);
    },
  );
}
