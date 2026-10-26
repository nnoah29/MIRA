import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_widget_from_html_core/flutter_widget_from_html_core.dart';
import 'package:html/dom.dart' as dom;
import 'package:provider/provider.dart';
import 'package:url_launcher/url_launcher.dart' show LaunchMode, launchUrl;

import '../data/models.dart';
import '../i18n/i18n.dart';
import '../state/image_policy.dart';
import '../state/store.dart';
import '../theme/app_theme.dart';
import 'body_skeleton.dart';
import 'bottom_baseline.dart';
import 'mail_widget_factory.dart';

/// Au-delà, l'analyse du HTML part dans un isolat (le surcoût ne vaut que pour les gros courriels).
const _asyncAbove = 30000;
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
  if (Platform.isAndroid) {
    return launchUrl(uri, mode: LaunchMode.externalApplication);
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
    final asyncBuild = msg.html!.length > _asyncAbove;
    if (!asyncBuild) _markReady();
    return _body = HtmlWidget(
      msg.html!,
      buildAsync: asyncBuild,
      onLoadingBuilder: asyncBuild
          ? (_, _, _) => _LoadMarker(onGone: _markReady)
          : null,
      textStyle: style,
      factoryBuilder: MailWidgetFactory.new,
      customWidgetBuilder: (e) =>
          e.localName == 'img' ? _image(e, remoteOk, style) : null,
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
    duration: const Duration(milliseconds: 260),
    curve: Curves.easeInOutCubic,
    alignment: Alignment.topCenter,
    child: Stack(
      clipBehavior: Clip.hardEdge,
      children: [
        AnimatedOpacity(
          opacity: _ready ? 1 : 0,
          curve: Curves.easeInOut,
          duration: const Duration(milliseconds: 320),
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
    final html = _htmlBody(p, remoteOk, onPaper);
    final card = onPaper
        ? Container(
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: _paper,
              borderRadius: BorderRadius.circular(8),
            ),
            child: html,
          )
        : html;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        if (msg.blocked.isNotEmpty && action == ImageAction.ask)
          _RemoteBanner(msg: msg),
        _faded(card),
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

  Widget? _image(dom.Element e, bool remoteOk, TextStyle style) {
    final width = _length(e, 'width');
    final height = _length(e, 'height');
    if (_isTracker(width, height)) return const SizedBox.shrink();
    final alt = (e.attributes['alt'] ?? '').trim();
    final image = _provider((e.attributes['src'] ?? '').trim(), remoteOk);
    if (image == null && alt.isEmpty) return const SizedBox.shrink();
    final fallback = Text(alt, style: style.copyWith(fontSize: 13));
    final picture = image == null
        ? fallback
        : BottomBaseline(
            child: _rounded(
              e,
              Image(
                image: image,
                width: width,
                height: height,
                fit: width != null && height != null
                    ? BoxFit.fill
                    : BoxFit.contain,
                errorBuilder: (_, _, _) =>
                    alt.isEmpty ? const SizedBox.shrink() : fallback,
              ),
              width,
            ),
          );
    final loose = Align(
      alignment: _alignmentOf(e),
      widthFactor: 1,
      heightFactor: 1,
      child: picture,
    );
    return _isBlock(e) ? loose : InlineCustomWidget(child: loose);
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

/// Pixel espion ou espaceur : 2 px ou moins sur un côté, jamais affiché.
bool _isTracker(double? width, double? height) =>
    (width != null && width <= 2) || (height != null && height <= 2);

final _displayBlock = RegExp(
  r'(?<![-\w])display\s*:\s*block',
  caseSensitive: false,
);

/// Une image est en ligne, comme dans un navigateur, sauf `display:block`.
bool _isBlock(dom.Element e) =>
    _displayBlock.hasMatch(e.attributes['style'] ?? '');

({double value, bool percent})? _measure(String? raw) {
  final found = RegExp(r'^\s*([\d.]+)\s*(px|%)?\s*$').firstMatch(raw ?? '');
  final value = double.tryParse(found?.group(1) ?? '');
  return value == null ? null : (value: value, percent: found?.group(2) == '%');
}

String? _declared(dom.Element e, String name) => RegExp(
  '(?<![-\\w])$name\\s*:\\s*([^;]+)',
  caseSensitive: false,
).firstMatch(e.attributes['style'] ?? '')?.group(1);

/// Dimension voulue de l'image : le style en ligne l'emporte sur l'attribut. Un pourcentage de largeur se calcule sur la largeur déclarée du conteneur ; sans conteneur déclaré, l'image garde sa taille naturelle.
double? _length(dom.Element e, String name) {
  final measure = _measure(_declared(e, name)) ?? _measure(e.attributes[name]);
  if (measure == null) return null;
  if (!measure.percent) return measure.value;
  final base = name == 'width' ? _containerWidth(e) : null;
  return base == null ? null : base * measure.value / 100;
}

/// Largeur en pixels déclarée par le plus proche ancêtre qui en a une (cellule, table, bloc).
double? _containerWidth(dom.Element e) {
  for (var parent = e.parent; parent != null; parent = parent.parent) {
    final width =
        _measure(_declared(parent, 'width')) ??
        _measure(parent.attributes['width']);
    if (width != null && !width.percent) return width.value;
  }
  return null;
}

final _textAlignRule = RegExp(
  r'(?<![-\w])text-align\s*:\s*(\w+)',
  caseSensitive: false,
);

/// Place l'image comme un navigateur : selon l'`align` ou le `text-align` du premier conteneur qui en porte un, à gauche sinon.
Alignment _alignmentOf(dom.Element e) {
  for (var parent = e.parent; parent != null; parent = parent.parent) {
    final style = _textAlignRule.firstMatch(parent.attributes['style'] ?? '');
    final named = parent.attributes['align'] ?? style?.group(1);
    final value = parent.localName == 'center' ? 'center' : named;
    switch (value?.toLowerCase()) {
      case 'center':
        return Alignment.topCenter;
      case 'right':
        return Alignment.topRight;
      case 'left':
        return Alignment.topLeft;
    }
  }
  return Alignment.topLeft;
}

final _radiusRule = RegExp(
  r'(?<![-\w])border-radius\s*:\s*([\d.]+)\s*(%|px)',
  caseSensitive: false,
);

/// Applique l'arrondi du style en ligne : cercle à partir de 50 % (ou d'un rayon couvrant l'image), coins en pixels sinon.
Widget _rounded(dom.Element e, Widget image, double? width) {
  final match = _radiusRule.firstMatch(e.attributes['style'] ?? '');
  final value = double.tryParse(match?.group(1) ?? '');
  if (match == null || value == null || value <= 0) return image;
  final percent = match.group(2) == '%';
  final circle = percent ? value >= 50 : value >= (width ?? 2000) / 2;
  if (circle) return ClipOval(child: image);
  if (percent) return image;
  return ClipRRect(borderRadius: BorderRadius.circular(value), child: image);
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
