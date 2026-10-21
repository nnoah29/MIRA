import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:webview_all/webview_all.dart';

import '../engine/web_document.dart';
import 'html_body.dart';

/// Canal par lequel notre script de page parle à Flutter.
const _channel = 'MiraMail';

/// Hauteur avant la première mesure, et bornes (une vue GTK ne peut pas dépasser 32 767 px).
const _initialHeight = 120.0;
const _maxHeight = 30000.0;

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
  function measure() {
    post({height: Math.ceil(document.documentElement.getBoundingClientRect().height)});
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
  window.addEventListener('load', measure);
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

class _WebMailBodyState extends State<WebMailBody> {
  final _controller = WebViewController();
  late final Future<void> _ready = _prepare();
  double _height = _initialHeight;
  double? _width;

  @override
  void initState() {
    super.initState();
    _load();
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
  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, box) {
      _remeasureOnResize(box.maxWidth);
      return SizedBox(
        height: _height,
        child: WebViewWidget(controller: _controller),
      );
    },
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
