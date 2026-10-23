import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/inline_images.dart';

Map<String, dynamic> _att(String cid, int size) => {
  'id': cid,
  'content_id': cid,
  'size': size,
};

void main() {
  test(
    'les images incorporées d\'un message tiennent dans un budget total',
    () {
      final all = [for (var i = 0; i < 6; i++) _att('c$i', 3 * 1024 * 1024)];
      final kept = withinInlineBudget(all);
      final total = kept.fold<num>(0, (s, a) => s + (a['size'] as num));
      expect(total, lessThanOrEqualTo(inlineBudget));
      expect(kept.map((a) => a['id']), ['c0', 'c1']);
    },
  );

  test(
    'une image qui dépasse le budget restant n\'empêche pas les petites',
    () {
      final all = [
        _att('a', 5 * 1024 * 1024),
        _att('b', 4 * 1024 * 1024),
        _att('c', 10),
      ];
      expect(withinInlineBudget(all).map((a) => a['id']), ['a', 'c']);
    },
  );
}
