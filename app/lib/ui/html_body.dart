import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_widget_from_html_core/flutter_widget_from_html_core.dart';
import 'package:html/dom.dart' as dom;
import 'package:provider/provider.dart';

import '../data/models.dart';
import '../i18n/i18n.dart';
import '../state/image_policy.dart';
import '../state/store.dart';
import '../theme/app_theme.dart';
import 'body_skeleton.dart';

const _paper = Color(0xFFFFFFFF);
const _ink = Color(0xFF1A1A1A);

/// Le HTML impose-t-il ses propres couleurs (il doit alors rester sur fond clair) ?
bool imposesColors(String html) => RegExp(
  r'(bgcolor|color\s*[:=]|background)',
  caseSensitive: false,
).hasMatch(html);

/// Ouvre une adresse web ou mail dans le navigateur du système (schémas sûrs seulement).
Future<bool> openLink(String url) async {
  final uri = Uri.tryParse(url);
  if (uri == null || !const {'http', 'https', 'mailto'}.contains(uri.scheme)) {
    return false;
  }
  if (!Platform.isLinux) return false;
  await Process.run('xdg-open', [url]);
  return true;
}

/// Corps HTML d'un message : tableaux, images incorporées, images distantes si autorisées.
class MailHtml extends StatefulWidget {
  final Msg msg;
  final TextStyle base;
  const MailHtml({super.key, required this.msg, required this.base});

  @override
  State<MailHtml> createState() => _MailHtmlState();
}

class _MailHtmlState extends State<MailHtml> {
  Object? _key;
  Widget? _body;
  bool _ready = false;

  Msg get msg => widget.msg;

  /// Construit l'arbre HTML seulement si son contenu ou son contexte a changé (coûteux).
  Widget _htmlBody(Pal p, bool remoteOk, bool onPaper) {
    final key = Object.hash(
      msg.html,
      remoteOk,
      onPaper,
      p.accent,
      msg.inline.length,
      widget.base.color,
    );
    if (key == _key && _body != null) return _body!;
    final style = onPaper
        ? widget.base.copyWith(color: _ink, fontSize: 14, height: 1.5)
        : widget.base;
    _key = key;
    _ready = false;
    return _body = HtmlWidget(
      msg.html!,
      buildAsync: true,
      onLoadingBuilder: (_, _, _) => _LoadMarker(onGone: _markReady),
      textStyle: style,
      customWidgetBuilder: (e) =>
          e.localName == 'img' ? _image(e, remoteOk) : null,
      customStylesBuilder: (e) => _styleOf(e, p, onPaper),
      onTapUrl: openLink,
    );
  }

  void _markReady() {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) setState(() => _ready = true);
    });
  }

  /// Le HTML apparaît en fondu au-dessus d'un squelette, et la hauteur s'anime.
  Widget _faded(Widget body) => AnimatedSize(
    duration: const Duration(milliseconds: 180),
    curve: Curves.easeOutCubic,
    alignment: Alignment.topCenter,
    child: Stack(
      clipBehavior: Clip.hardEdge,
      children: [
        AnimatedOpacity(
          opacity: _ready ? 1 : 0,
          duration: const Duration(milliseconds: 160),
          child: body,
        ),
        if (!_ready)
          const Positioned(top: 0, left: 0, right: 0, child: BodySkeleton()),
      ],
    ),
  );

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final action = context.select<MailStore, ImageAction>(
      (st) => st.imageActionFor(msg.from.email),
    );
    final remoteOk = switch (action) {
      ImageAction.always => true,
      ImageAction.never => false,
      ImageAction.ask => msg.blocked.isEmpty,
    };
    final onPaper = imposesColors(msg.html!);
    final body = _faded(_htmlBody(p, remoteOk, onPaper));
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        if (msg.blocked.isNotEmpty && action == ImageAction.ask)
          _RemoteBanner(msg: msg),
        if (onPaper)
          Container(
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: _paper,
              borderRadius: BorderRadius.circular(8),
            ),
            child: body,
          )
        else
          body,
      ],
    );
  }

  Map<String, String>? _styleOf(dom.Element e, Pal p, bool onPaper) {
    if (e.localName == 'a' && !onPaper) {
      return {'color': _hex(p.accent)};
    }
    if (e.localName == 'blockquote') {
      return {
        'border-left': '3px solid ${_hex(p.accent)}',
        'margin': '8px 0',
        'padding-left': '12px',
      };
    }
    return null;
  }

  Widget? _image(dom.Element e, bool remoteOk) {
    final src = (e.attributes['src'] ?? '').trim();
    final width = double.tryParse(e.attributes['width'] ?? '');
    final image = _provider(src, remoteOk);
    if (image == null) return const SizedBox.shrink();
    return ConstrainedBox(
      constraints: BoxConstraints(maxWidth: width ?? double.infinity),
      child: Image(
        image: image,
        fit: BoxFit.contain,
        errorBuilder: (_, _, _) => const Icon(Icons.broken_image_outlined),
      ),
    );
  }

  ImageProvider? _provider(String src, bool remoteOk) {
    final lower = src.toLowerCase();
    if (lower.startsWith('cid:')) {
      final bytes =
          msg.inline[lower.substring(4).replaceAll(RegExp(r'^<|>$'), '')];
      return bytes == null ? null : MemoryImage(bytes);
    }
    if (lower.startsWith('data:image/')) {
      final data = Uri.tryParse(src)?.data;
      return data == null ? null : MemoryImage(data.contentAsBytes());
    }
    final remote = lower.startsWith('http://') || lower.startsWith('https://');
    if (remote && remoteOk) {
      return NetworkImage(src);
    }
    return null;
  }
}

String _hex(Color c) =>
    '#${(c.toARGB32() & 0xFFFFFF).toRadixString(16).padLeft(6, '0')}';

/// Bandeau « images distantes bloquées » avec le choix de les afficher.
class _RemoteBanner extends StatelessWidget {
  final Msg msg;
  const _RemoteBanner({required this.msg});

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final store = context.read<MailStore>();
    return Container(
      margin: const EdgeInsets.only(bottom: 10),
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: BoxDecoration(
        color: p.surface2,
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: p.line),
      ),
      child: Wrap(
        crossAxisAlignment: WrapCrossAlignment.center,
        spacing: 8,
        children: [
          Text(
            t('Images distantes bloquées pour votre confidentialité'),
            style: TextStyle(color: p.muted, fontSize: 13),
          ),
          TextButton(
            onPressed: () => store.allowRemote(msg),
            child: Text(t('Afficher')),
          ),
          TextButton(
            onPressed: () => store.allowRemote(msg, sender: true),
            child: Text(t('Toujours pour cet expéditeur')),
          ),
        ],
      ),
    );
  }
}

/// Signale à son parent qu'il a disparu (le HTML est construit) ; occupe la place du squelette.
class _LoadMarker extends StatefulWidget {
  final VoidCallback onGone;
  const _LoadMarker({required this.onGone});

  @override
  State<_LoadMarker> createState() => _LoadMarkerState();
}

class _LoadMarkerState extends State<_LoadMarker> {
  @override
  void dispose() {
    widget.onGone();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => const SizedBox(height: 96);
}
