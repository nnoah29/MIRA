import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:webview_all/webview_all.dart';

import '../engine/web_document.dart';
import 'html_body.dart';

/// Corps d'un message affiché par un vrai moteur web (WebKitGTK sur Linux) : le HTML reçu, tel quel, sans JavaScript, sous une politique de contenu stricte.
class WebMailBody extends StatefulWidget {
  final String html;
  final Map<String, Uint8List> inline;
  final bool remoteImages;
  final double height;
  const WebMailBody({
    super.key,
    required this.html,
    required this.inline,
    required this.remoteImages,
    required this.height,
  });

  @override
  State<WebMailBody> createState() => _WebMailBodyState();
}

class _WebMailBodyState extends State<WebMailBody> {
  late final WebViewController _controller;

  @override
  void initState() {
    super.initState();
    _controller = WebViewController()
      ..setJavaScriptMode(JavaScriptMode.disabled)
      ..setBackgroundColor(Colors.white)
      ..setNavigationDelegate(
        NavigationDelegate(onNavigationRequest: _onNavigate),
      );
    _load();
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

  void _load() {
    _controller.loadHtmlString(
      webDocument(
        widget.html,
        remoteImages: widget.remoteImages,
        inline: widget.inline,
      ),
    );
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

  @override
  Widget build(BuildContext context) => SizedBox(
    height: widget.height,
    child: ClipRRect(
      borderRadius: BorderRadius.circular(8),
      child: WebViewWidget(controller: _controller),
    ),
  );
}
