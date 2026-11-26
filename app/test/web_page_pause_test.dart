import 'package:flutter_test/flutter_test.dart';
import 'package:mira/ui/web_page_pause.dart';

void main() {
  test('le script définit le point d\'entrée appelé par l\'app', () {
    expect(webPagePauseScript, contains('window.__miraPause = function'));
  });

  test('les images sont figées puis rendues, les animations pausées puis relancées', () {
    expect(webPagePauseScript, contains('drawImage'));
    expect(webPagePauseScript, contains("img.style.display = 'none'"));
    expect(webPagePauseScript, contains('animation.pause()'));
    expect(webPagePauseScript, contains('animation.play()'));
  });

  test('le script est injecté une seule fois par page', () {
    expect(webPagePauseScript, contains('if (window.__miraPause) return;'));
  });
}
