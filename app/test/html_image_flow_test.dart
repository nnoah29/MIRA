import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/data/models.dart';
import 'package:mira/state/store.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/html_body.dart';
import 'package:provider/provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

const _data =
    'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAASwAAAB4CAIAAADHd1h3AAAC4UlEQVR4nO3WQQ0AIRTEUOQgcSUia2X0MC+pgs5v4NzvgQEGQgOHfQYYaA2I0AkyIMJaARjwEjoCBqYN+I72G2DcgAj7DTBuQIT9Bhg3IMJ+A4wbEGG/AcYNiLDfAOMGRNhvgHEDIuw3wLgBEfYbYNyACPsNMG5AhP0GGDcgwn4DjBsQYb8Bxg2IsN8A4wZE2G+AcQMi7DfAuAER9htg3IAI+w0wbkCE/QYYNyDCfgOMGxBhvwHGDYiw3wDjBkTYb4BxAyLsN8C4ARH2G2DcgAj7DTBuQIT9Bhg3IMJ+A4wbEGG/AcYNiLDfAOMGRNhvgHEDIuw3wLgBEfYbYNyACPsNMG5AhP0GGDcgwn4DjBsQYb8Bxg2IsN8A4wZE2G+AcQMi7DfAuAER9htg3IAI+w0wbkCE/QYYNyDCfgOMGxBhvwHGDYiw3wDjBkTYb4BxAyLsN8C4ARH2G2DcgAj7DTBuQIT9Bhg3IMJ+A4wbEGG/AcYNiLDfAOMGRNhvgHEDIuw3wLgBEfYbYNyACPsNMG5AhP0GGDcgwn4DjBsQYb8Bxg2IsN8A4wZE2G+AcQMi7DfAuAER9htg3IAI+w0wbkCE/QYYNyDCfgOMGxBhvwHGDYiw3wDjBkTYb4BxAyLsN8C4ARH2G2DcgAj7DTBuQIT9Bhg3IMJ+A4wbEGG/AcYNiLDfAOMGRNhvgHEDIuw3wLgBEfYbYNyACPsNMG5AhP0GGDcgwn4DjBsQYb8Bxg2IsN8A4wZE2G+AcQMi7DfAuAER9htg3IAI+w0wbkCE/QYYNyDCfgOMGxBhvwHGDYiw3wDjBkTYb4BxAyLsN8C4ARH2G2DcgAj7DTBuQIT9Bhg3IMJ+A4wbEGG/AcYNiLDfAOMGRNhvgHEDIuw3wLgBEfYbYNyACPsNMG5AhP0GGDcgwn4DjBsQYb8Bxg2IsN8A4wZE2G+AcQMi7DfAuAER9htg3IAI+w0wbkCE/Qa42xJ+WZ/MjDH7k/QAAAAASUVORK5CYII=';

/// Rend le HTML (où `IMG` devient une image de 300×120 px) et rend la boîte de chaque image.
Future<List<Rect>> _rectsOf(WidgetTester tester, String html) async {
  SharedPreferences.setMockInitialValues({});
  final store = MailStore(await SharedPreferences.getInstance());
  final msg = Msg(from: const Person('L', 'l@x.org'), date: 0, body: const [])
    ..html = html.replaceAll('IMG', '<img src="$_data"')
    ..blocked = const []
    ..inline = {};
  await tester.binding.setSurfaceSize(const Size(900, 700));
  addTearDown(() => tester.binding.setSurfaceSize(null));
  await tester.pumpWidget(
    ChangeNotifierProvider.value(
      value: store,
      child: MaterialApp(
        theme: buildTheme(false),
        home: Scaffold(
          body: SingleChildScrollView(
            child: MailHtml(msg: msg, base: const TextStyle(fontSize: 16)),
          ),
        ),
      ),
    ),
  );
  await tester.pump();
  await tester.runAsync(
    () => Future<void>.delayed(const Duration(milliseconds: 300)),
  );
  await tester.pump(const Duration(milliseconds: 600));
  final images = find.byType(Image);
  return [
    for (var i = 0; i < images.evaluate().length; i++)
      tester.getRect(images.at(i)),
  ];
}

void main() {
  testWidgets('des images en ligne se placent côte à côte', (tester) async {
    final rects = await _rectsOf(
      tester,
      '<p><a href="a">IMG width="40"></a> <a href="b">IMG width="40"></a> <a href="c">IMG width="40"></a></p>',
    );
    expect(rects, hasLength(3));
    expect((rects[1].top - rects[0].top).abs(), lessThan(2));
    expect(rects[1].left, greaterThan(rects[0].right - 1));
    expect(rects[2].left, greaterThan(rects[1].right - 1));
  });

  testWidgets('des images display:block s’empilent', (tester) async {
    final rects = await _rectsOf(
      tester,
      'IMG width="40" style="display:block"> IMG width="40" style="display:block">',
    );
    expect(rects[1].top, greaterThan(rects[0].bottom - 1));
  });

  testWidgets(
    'un pourcentage se calcule sur la largeur déclarée du conteneur',
    (tester) async {
      final rects = await _rectsOf(
        tester,
        '<table><tr><td width="200">IMG width="50%"></td></tr></table>',
      );
      expect(rects.single.width, 100);
    },
  );

  testWidgets(
    'sans conteneur déclaré, un pourcentage laisse la taille naturelle',
    (tester) async {
      final rects = await _rectsOf(tester, '<div>IMG width="50%"></div>');
      expect(rects.single.size, const Size(300, 120));
    },
  );

  testWidgets('une largeur en pixels plus grande que l’image l’agrandit', (
    tester,
  ) async {
    final rects = await _rectsOf(
      tester,
      'IMG width="500" style="display:block">',
    );
    expect(rects.single.width, 500);
  });

  testWidgets('largeur et hauteur imposées donnent exactement cette boîte', (
    tester,
  ) async {
    final rects = await _rectsOf(tester, 'IMG width="100" height="100">');
    expect(rects.single.size, const Size(100, 100));
  });
}
