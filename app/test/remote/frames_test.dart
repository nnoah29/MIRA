import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/remote/frame_builders.dart';
import 'package:mira/engine/remote/frames.dart';

/// Vecteurs partagés avec le crate Rust `mira-protocol` : `{name, frame, hex}`.
List<Map<String, dynamic>> goldenVectors() {
  final file = File('../engine/crates/mira-protocol/tests/golden/frames.json');
  return (jsonDecode(file.readAsStringSync()) as List)
      .cast<Map<String, dynamic>>();
}

Uint8List fromHex(String hex) => Uint8List.fromList([
  for (var i = 0; i < hex.length; i += 2)
    int.parse(hex.substring(i, i + 2), radix: 16),
]);

/// La trame `name` des vecteurs partagés.
Map<String, dynamic> golden(String name) =>
    goldenVectors().firstWhere((v) => v['name'] == name)['frame']
        as Map<String, dynamic>;

/// Trame de test dont le corps JSON fait au moins `size` caractères.
Map<String, dynamic> bigFrame(int size) => {
  'type': 'result',
  'id': 1,
  'envelope': {'pad': 'x' * size},
};

void main() {
  test('dart_encodes_every_golden_frame_to_the_rust_bytes', () {
    for (final v in goldenVectors()) {
      final bytes = encodeFrame(v['frame'] as Map<String, dynamic>);
      expect(bytes, fromHex(v['hex'] as String), reason: '${v['name']}');
    }
  });

  test('dart_decodes_every_golden_hex', () {
    for (final v in goldenVectors()) {
      final frames = FrameDecoder().add(fromHex(v['hex'] as String));
      expect(frames, [v['frame']], reason: '${v['name']}');
    }
  });

  test('a_frame_split_one_byte_at_a_time_decodes_once', () {
    final decoder = FrameDecoder();
    final bytes = encodeFrame(golden('call'));
    final out = <Map<String, dynamic>>[];
    for (final b in bytes) {
      out.addAll(decoder.add([b]));
    }
    expect(out, [golden('call')]);
  });

  test('two_frames_in_one_chunk_decode_in_order', () {
    final bytes = Uint8List.fromList([
      ...encodeFrame(golden('ping')),
      ...encodeFrame(golden('cancel')),
    ]);
    expect(FrameDecoder().add(bytes), [golden('ping'), golden('cancel')]);
  });

  test('a_partial_header_yields_nothing_until_complete', () {
    final decoder = FrameDecoder();
    final bytes = encodeFrame(golden('pong'));
    expect(decoder.add(bytes.sublist(0, 2)), isEmpty);
    expect(decoder.add(bytes.sublist(2, 5)), isEmpty);
    expect(decoder.add(bytes.sublist(5)), [golden('pong')]);
  });

  test('an_announced_size_above_the_maximum_fails_from_the_header_alone', () {
    final header = ByteData(4)..setUint32(0, 1024 * 1024);
    expect(
      () => FrameDecoder(max: 1024).add(header.buffer.asUint8List()),
      throwsA(
        isA<ProtocolException>().having((e) => e.reason, 'reason', 'too_large'),
      ),
    );
  });

  test('a_zero_length_frame_fails', () {
    expect(
      () => FrameDecoder().add([0, 0, 0, 0]),
      throwsA(
        isA<ProtocolException>().having((e) => e.reason, 'reason', 'empty'),
      ),
    );
  });

  test('a_body_that_is_not_an_object_fails', () {
    final body = utf8.encode('[1,2]');
    final bytes = Uint8List.fromList([0, 0, 0, body.length, ...body]);
    expect(
      () => FrameDecoder().add(bytes),
      throwsA(
        isA<ProtocolException>().having((e) => e.reason, 'reason', 'malformed'),
      ),
    );
  });

  test('encode_refuses_a_body_above_the_maximum', () {
    expect(
      () => encodeFrame(bigFrame(2000), max: 1024),
      throwsA(
        isA<ProtocolException>().having((e) => e.reason, 'reason', 'too_large'),
      ),
    );
  });

  test('builders_produce_the_documented_fields', () {
    String s(Map<String, dynamic> m) => jsonEncode(m);
    expect(
      s(helloFrame(clientId: 'c1', clientName: 'mira-app', token: 't')),
      s(golden('hello')),
    );
    expect(
      s(
        callFrame(1, {
          'domain': 'account',
          'request': {'op': 'list_accounts'},
        }),
      ),
      s(golden('call')),
    );
    expect(s(openEventsFrame(2, credit: 8)), s(golden('open_events')));
    expect(
      s(openEventsFrame(2, after: 41, credit: 256)),
      s(golden('open_events_after')),
    );
    expect(
      s(openChatFrame(3, {'mode': 'x'}, credit: 8)),
      s(golden('open_chat')),
    );
    expect(s(getOriginalHtmlFrame(4, 'm1')), s(golden('get_original_html')));
    expect(s(hostOpShutdownFrame(5)), s(golden('host_op')));
    expect(s(creditFrame(2, 4)), s(golden('credit')));
    expect(s(cancelFrame(3)), s(golden('cancel')));
    expect(s(pingFrame(7)), s(golden('ping')));
  });
}
