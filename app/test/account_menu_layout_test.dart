import 'dart:ui';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/data/models.dart';
import 'package:mira/ui/overlays.dart';

import 'app_test.dart' show pumpApp, teardown;

/// Cadre de l'avatar d'un compte dans le menu (le dernier trouvé : le menu est au-dessus de la barre).
Rect avatarOf(WidgetTester tester, Account a) {
  final avatar = find.byWidgetPredicate(
    (w) => w is AccountAvatar && w.a.email == a.email,
  );
  return tester.getRect(avatar.last);
}

/// Cadre du bloc de texte d'un compte : son nom, au-dessus de son adresse, dans la même colonne.
Rect textOf(WidgetTester tester, Account a) {
  final email = tester.getRect(find.text(a.email).last);
  final names = find
      .text(a.name)
      .evaluate()
      .map((e) => tester.getRect(find.byElementPredicate((x) => x == e)));
  final name = names.firstWhere(
    (r) => r.left == email.left && r.bottom <= email.top + 0.5,
  );
  return name.expandToInclude(email);
}

void main() {
  testWidgets('menu des comptes : chaque avatar est centré sur son texte', (
    tester,
  ) async {
    final s = await pumpApp(tester);
    await tester.tap(find.byTooltip('Changer de compte'));
    await tester.pump(const Duration(milliseconds: 200));
    for (final a in s.accounts) {
      final gap = avatarOf(tester, a).center.dy - textOf(tester, a).center.dy;
      expect(gap.abs(), lessThan(1.5), reason: 'avatar de ${a.email} décalé');
    }
    await teardown(tester, s);
  });
}
