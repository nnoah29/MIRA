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

/// Rend une image de 300×120 px avec la balise donnée et rend la boîte obtenue.
Future<Size> _sizeOf(WidgetTester tester, String attributes) async {
  SharedPreferences.setMockInitialValues({});
  final store = MailStore(await SharedPreferences.getInstance());
  final msg = Msg(from: const Person('L', 'l@x.org'), date: 0, body: const [])
    ..html = '<p><img src="$_data" $attributes></p>'
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
  return tester.getSize(find.byType(Image));
}

void main() {
  testWidgets('une image sans dimension garde sa taille naturelle', (
    tester,
  ) async {
    expect(await _sizeOf(tester, ''), const Size(300, 120));
  });

  testWidgets('la largeur de l’attribut borne l’image', (tester) async {
    expect((await _sizeOf(tester, 'width="150"')).width, 150);
  });

  testWidgets('la largeur du style borne l’image, sans attribut', (
    tester,
  ) async {
    final size = await _sizeOf(tester, 'style="width:100px"');
    expect(size.width, 100);
    expect(size.height, 40);
  });

  testWidgets('le style l’emporte sur l’attribut', (tester) async {
    final size = await _sizeOf(tester, 'width="250" style="width:100px"');
    expect(size.width, 100);
  });

  testWidgets('la hauteur du style borne aussi l’image', (tester) async {
    final size = await _sizeOf(tester, 'style="height:30px"');
    expect(size.height, 30);
    expect(size.width, 75);
  });
}
