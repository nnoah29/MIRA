import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/state/unsubscribe.dart';

void main() {
  test('les adresses locales, privées et réservées ne sont pas publiques', () {
    for (final ip in [
      '127.0.0.1',
      '10.0.0.5',
      '172.16.3.1',
      '192.168.0.2',
      '169.254.169.254',
      '0.0.0.0',
      '100.64.0.1',
      '::1',
      'fd00::1',
      'fe80::1',
      '::ffff:192.168.1.1',
    ]) {
      expect(isPublicAddress(InternetAddress(ip)), isFalse, reason: ip);
    }
    for (final ip in ['8.8.8.8', '172.32.0.1', '2606:4700:4700::1111']) {
      expect(isPublicAddress(InternetAddress(ip)), isTrue, reason: ip);
    }
  });

  test('une adresse qui se résout en privé est refusée sans requête', () async {
    final server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
    var hits = 0;
    server.listen((r) {
      hits++;
      r.response.close();
    });
    final ok = await Unsubscriber().postOneClick(
      Uri.parse('https://localhost:${server.port}/u'),
    );
    await server.close(force: true);
    expect(ok, isFalse);
    expect(hits, 0);
  });

  test('une redirection n\'est pas suivie', () async {
    final server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
    final paths = <String>[];
    server.listen((r) {
      paths.add(r.uri.path);
      r.response
        ..statusCode = 302
        ..headers.set('location', '/ailleurs')
        ..close();
    });
    final u = Unsubscriber()..allowAddress = (_) => true;
    final ok = await u.postHttp(Uri.parse('http://127.0.0.1:${server.port}/u'));
    await server.close(force: true);
    expect(ok, isFalse);
    expect(paths, ['/u']);
  });

  test('une réponse géante est abandonnée', () async {
    final server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
    server.listen((r) async {
      r.response.statusCode = 200;
      for (var i = 0; i < 200; i++) {
        r.response.add(List.filled(64 * 1024, 65));
      }
      await r.response.close().catchError((_) {});
    });
    final u = Unsubscriber()..allowAddress = (_) => true;
    final ok = await u.postHttp(Uri.parse('http://127.0.0.1:${server.port}/u'));
    await server.close(force: true);
    expect(ok, isFalse);
  });
}
