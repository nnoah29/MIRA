import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/remote/byte_channel.dart';

import 'support/memory_channel.dart';

/// Premier bloc d'octets reçu par un canal.
Future<List<int>> firstBytes(ByteChannel c) => c.incoming.first;

void main() {
  test(
    'memory_channel_delivers_bytes_both_ways_and_close_propagates',
    () async {
      final (a, b) = MemoryChannel.pair();
      a.send([1, 2, 3]);
      expect(await firstBytes(b), [1, 2, 3]);
      b.send([9]);
      expect(await firstBytes(a), [9]);
      await a.close();
      await b.done;
      await a.done;
      b.send([5]);
    },
  );

  test('tcp_channel_exchanges_bytes_with_a_loopback_server', () async {
    final server = await ServerSocket.bind('127.0.0.1', 0);
    server.listen((s) => s.listen((data) => s.add(data.reversed.toList())));
    final channel = await SocketChannel.connectTcp('127.0.0.1', server.port);
    channel.send([1, 2, 3]);
    expect(await firstBytes(channel), [3, 2, 1]);
    await channel.close();
    await server.close();
  });

  test('unix_channel_exchanges_bytes_with_a_unix_server', () async {
    final dir = Directory.systemTemp.createTempSync('mira-chan');
    final path = '${dir.path}/s.sock';
    final server = await ServerSocket.bind(
      InternetAddress(path, type: InternetAddressType.unix),
      0,
    );
    server.listen((s) => s.listen((data) => s.add([...data, 0])));
    final channel = await SocketChannel.connectUnix(path);
    channel.send([7]);
    expect(await firstBytes(channel), [7, 0]);
    await channel.close();
    await server.close();
    dir.deleteSync(recursive: true);
  }, testOn: 'linux');

  test('connecting_to_a_missing_unix_socket_fails_with_a_socket_exception', () {
    expect(
      SocketChannel.connectUnix(
        '/tmp/mira-absent-${DateTime.now().microsecondsSinceEpoch}.sock',
      ),
      throwsA(isA<SocketException>()),
    );
  }, testOn: 'linux');

  test('connecting_to_a_closed_tcp_port_fails', () async {
    final probe = await ServerSocket.bind('127.0.0.1', 0);
    final port = probe.port;
    await probe.close();
    expect(
      SocketChannel.connectTcp('127.0.0.1', port),
      throwsA(isA<SocketException>()),
    );
  });

  test('closing_completes_done_and_a_second_close_is_harmless', () async {
    final server = await ServerSocket.bind('127.0.0.1', 0);
    server.listen((s) => s.listen((_) {}));
    final channel = await SocketChannel.connectTcp('127.0.0.1', server.port);
    await channel.close();
    await channel.done;
    await channel.close();
    channel.send([1]);
    await server.close();
  });

  test('the_remote_end_closing_completes_done', () async {
    final server = await ServerSocket.bind('127.0.0.1', 0);
    server.listen((s) => s.destroy());
    final channel = await SocketChannel.connectTcp('127.0.0.1', server.port);
    await channel.done.timeout(const Duration(seconds: 3));
    await server.close();
  });
}
