import 'dart:async';
import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:webview_all/webview_all.dart';

import '../engine/web_document.dart';
import 'body_skeleton.dart';
import 'html_body.dart';

/// Canal par lequel notre script de page parle à Flutter.
const _channel = 'MiraMail';

/// Hauteur avant la première mesure, et bornes (une vue GTK ne peut pas dépasser 32 767 px).
const _initialHeight = 120.0;
const _maxHeight = 30000.0;

/// Dévoilement de la page, de haut en bas, une fois chargée (seule animation que la vue native sait suivre : une découpe rectangulaire).
const _curtainDuration = Duration(milliseconds: 260);

/// Au-delà, la page est dévoilée même si son chargement n'est pas fini (images distantes lentes).
const _revealAfter = Duration(seconds: 2);

/// Écart de hauteur ignoré : arrondis d'une mesure à l'autre, qui feraient osciller la vue.
const _jitter = 2.0;

/// Script injecté par l'app (pas par le courriel, dont les scripts restent bloqués par la politique de contenu) :
/// il signale la hauteur du document et renvoie la molette verticale au lecteur.
const _pageScript =
    '''
(function() {
  var handler = window.webkit && window.webkit.messageHandlers && window.webkit.messageHandlers.$_channel;
  if (!handler) return;
  function post(message) { try { handler.postMessage(JSON.stringify(message)); } catch (_) {} }
  function measure(loaded) {
    post({height: Math.ceil(document.documentElement.getBoundingClientRect().height), loaded: loaded === true});
  }
  window.addEventListener('wheel', function(event) {
    if (event.ctrlKey || Math.abs(event.deltaX) > Math.abs(event.deltaY)) return;
    event.preventDefault();
    var unit = event.deltaMode === 1 ? 40 : event.deltaMode === 2 ? window.innerHeight : 1;
    post({wheel: event.deltaY * unit});
  }, {passive: false, capture: true});
  document.addEventListener('DOMContentLoaded', function() {
    measure();
    if (window.ResizeObserver) {
      new ResizeObserver(measure).observe(document.documentElement);
      if (document.body) new ResizeObserver(measure).observe(document.body);
    }
  });
  window.addEventListener('load', function() { measure(true); });
  window.addEventListener('resize', measure);
})();
''';

/// Corps d'un message affiché par un vrai moteur web (WebKitGTK sur Linux) : le HTML reçu, tel quel, sous une politique de contenu stricte.
/// La vue prend la hauteur complète du courriel et défile avec le lecteur ; la molette au-dessus d'elle fait défiler le lecteur.
class WebMailBody extends StatefulWidget {
  final String html;
  final Map<String, Uint8List> inline;
  final bool remoteImages;
  const WebMailBody({
    super.key,
    required this.html,
    required this.inline,
    required this.remoteImages,
  });

  @override
  State<WebMailBody> createState() => _WebMailBodyState();
}

class _WebMailBodyState extends State<WebMailBody>
    with SingleTickerProviderStateMixin {
  final _controller = WebViewController();
  late final Future<void> _ready = _prepare();
  late final _curtain = AnimationController(
    vsync: this,
    duration: _curtainDuration,
  );
  Timer? _fallback;
  bool _revealed = false;
  double _height = _initialHeight;
  double? _width;

  @override
  void initState() {
    super.initState();
    _fallback = Timer(_revealAfter, _reveal);
    _load();
  }

  @override
  void dispose() {
    _fallback?.cancel();
    _curtain.dispose();
    super.dispose();
  }

  /// La page chargée et mesurée remplace le squelette en se dévoilant.
  void _reveal() {
    _fallback?.cancel();
    if (_revealed || !mounted) return;
    setState(() => _revealed = true);
    _curtain.forward();
  }

  /// JavaScript n'est activé que pour notre script : ceux du courriel sont interdits par la politique de contenu (`default-src 'none'`).
  Future<void> _prepare() async {
    await _controller.setJavaScriptMode(JavaScriptMode.unrestricted);
    await _controller.setBackgroundColor(Colors.white);
    await _controller.setNavigationDelegate(
      NavigationDelegate(onNavigationRequest: _onNavigate),
    );
    await _controller.addJavaScriptChannel(
      _channel,
      onMessageReceived: _onMessage,
    );
    await _controller.addUserScript(
      const WebViewUserScript(source: _pageScript),
    );
  }

  @override
  void didUpdateWidget(WebMailBody old) {
    super.didUpdateWidget(old);
    final changed =
        old.html != widget.html ||
        old.remoteImages != widget.remoteImages ||
        old.inline.length != widget.inline.length;
    if (changed) _load();
  }

  Future<void> _load() async {
    await _ready;
    await _controller.loadHtmlString(
      webDocument(
        widget.html,
        remoteImages: widget.remoteImages,
        inline: widget.inline,
      ),
    );
  }

  /// Messages du script de page : hauteur du document, ou défilement à transmettre au lecteur.
  void _onMessage(JavaScriptMessage message) {
    final Object? data;
    try {
      data = jsonDecode(message.message);
    } on FormatException {
      return;
    }
    if (data is! Map || !mounted) return;
    final height = data['height'];
    final wheel = data['wheel'];
    if (height is num && height > 0) {
      final next = height.toDouble().clamp(1.0, _maxHeight);
      if ((next - _height).abs() > _jitter) setState(() => _height = next);
      if (data['loaded'] == true) _reveal();
    } else if (wheel is num && wheel.isFinite) {
      Scrollable.maybeOf(context)?.position.pointerScroll(wheel.toDouble());
    }
  }

  /// Le document lui-même se charge ; tout lien cliqué part dans le navigateur du système.
  Future<NavigationDecision> _onNavigate(NavigationRequest request) async {
    final url = request.url;
    if (url.startsWith('about:') || url.startsWith('data:')) {
      return NavigationDecision.navigate;
    }
    await openLink(url);
    return NavigationDecision.prevent;
  }

  /// La vue native est posée par-dessus Flutter : une découpe non rectangulaire (coins arrondis) la ferait cacher par le plugin.
  /// Avant d'être dévoilée, elle garde sa vraie taille (WebKit met en page à la taille de la vue) mais n'est montrée que sur un pixel, sous le squelette.
  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, box) {
      _remeasureOnResize(box.maxWidth);
      return AnimatedBuilder(
        animation: _curtain,
        builder: (context, _) => SizedBox(
          height: _revealed ? _height : _initialHeight,
          child: Stack(
            children: [
              _revealed ? _view(_shownHeight(), box.maxWidth) : _view(1, 1),
              if (!_revealed) const Positioned.fill(child: BodySkeleton()),
            ],
          ),
        ),
      );
    },
  );

  double _shownHeight() =>
      (_height * Curves.easeOutCubic.transform(_curtain.value)).clamp(
        1.0,
        _height,
      );

  /// La vue à sa taille réelle, montrée seulement sur `shownWidth` × `shownHeight` en haut à gauche.
  Widget _view(double shownHeight, double shownWidth) => Align(
    alignment: Alignment.topLeft,
    child: SizedBox(
      width: shownWidth,
      height: shownHeight,
      child: ClipRect(
        child: OverflowBox(
          alignment: Alignment.topLeft,
          minWidth: _width ?? 1,
          maxWidth: _width ?? 1,
          minHeight: _height,
          maxHeight: _height,
          child: WebViewWidget(controller: _controller),
        ),
      ),
    ),
  );

  /// Une page ne mesure jamais moins que sa vue : quand la largeur change, la vue repart basse pour que la page donne sa vraie hauteur.
  void _remeasureOnResize(double width) {
    final previous = _width;
    _width = width;
    if (previous == null || previous == width) return;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) setState(() => _height = _initialHeight);
    });
  }
}
