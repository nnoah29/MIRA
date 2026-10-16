import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_widget_from_html_core/flutter_widget_from_html_core.dart';
import 'package:html/dom.dart' as dom;
import 'package:provider/provider.dart';

import '../data/models.dart';
import '../i18n/i18n.dart';
import '../state/store.dart';
import '../theme/app_theme.dart';

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
class MailHtml extends StatelessWidget {
  final Msg msg;
  final TextStyle base;
  const MailHtml({super.key, required this.msg, required this.base});

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final onPaper = imposesColors(msg.html!);
    final style = onPaper
        ? base.copyWith(color: _ink, fontSize: 14, height: 1.5)
        : base;
    final body = HtmlWidget(
      msg.html!,
      textStyle: style,
      customWidgetBuilder: (e) => e.localName == 'img' ? _image(e) : null,
      customStylesBuilder: (e) => _styleOf(e, p, onPaper),
      onTapUrl: openLink,
    );
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        if (msg.blocked.isNotEmpty) _RemoteBanner(msg: msg),
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

  Widget? _image(dom.Element e) {
    final src = (e.attributes['src'] ?? '').trim();
    final width = double.tryParse(e.attributes['width'] ?? '');
    final image = _provider(src);
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

  ImageProvider? _provider(String src) {
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
    if (lower.startsWith('http://') || lower.startsWith('https://')) {
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
