import 'dart:convert';
import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:pdfrx/pdfrx.dart';
import 'package:provider/provider.dart';

import '../data/documents.dart';
import '../data/models.dart';
import '../state/store.dart';
import '../theme/app_theme.dart';
import '../theme/icons.dart';
import 'widgets.dart';
import '../i18n/i18n.dart';

/// Ouvre la visionneuse de pièces jointes sur [list][[index]].
Future<void> showAttachmentViewer(
  BuildContext context,
  List<Attachment> list,
  int index,
) {
  return showGeneralDialog<void>(
    context: context,
    barrierDismissible: true,
    barrierLabel: t('Fermer'),
    barrierColor: const Color(0xCC000000),
    transitionDuration: const Duration(milliseconds: 180),
    pageBuilder: (_, _, _) => AttachmentViewer(list, index),
    transitionBuilder: (_, a, _, child) => FadeTransition(
      opacity: CurvedAnimation(parent: a, curve: Curves.easeOut),
      child: ScaleTransition(
        scale: Tween(
          begin: .97,
          end: 1.0,
        ).animate(CurvedAnimation(parent: a, curve: Curves.easeOutCubic)),
        child: child,
      ),
    ),
  );
}

class AttachmentViewer extends StatefulWidget {
  final List<Attachment> list;
  final int index;
  const AttachmentViewer(this.list, this.index, {super.key});
  @override
  State<AttachmentViewer> createState() => _AttachmentViewerState();
}

class _AttachmentViewerState extends State<AttachmentViewer> {
  late int i = widget.index;
  double zoom = 1;
  final tc = TransformationController();

  Attachment get a => widget.list[i];
  bool get multi => widget.list.length > 1;

  @override
  void dispose() {
    tc.dispose();
    super.dispose();
  }

  void _go(int d) {
    if (!multi) return;
    setState(() {
      i = (i + d) % widget.list.length;
      if (i < 0) i += widget.list.length;
      zoom = 1;
      tc.value = Matrix4.identity();
    });
  }

  void _zoom(double d) {
    setState(() {
      zoom = (zoom + d).clamp(.5, 3.0);
      tc.value = Matrix4.diagonal3Values(zoom, zoom, 1);
    });
  }

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final s = context.read<MailStore>();
    final size = MediaQuery.sizeOf(context);
    final narrow = size.width <= 760;
    final kind = kindOf(a);
    final realPdf = kind == AttKind.pdf && (a.id != null || a.path != null);
    final zoomable = kind != AttKind.other && kind != AttKind.text && !realPdf;

    Widget header = Container(
      padding: EdgeInsets.symmetric(horizontal: narrow ? 12 : 20, vertical: 12),
      decoration: BoxDecoration(
        border: Border(bottom: BorderSide(color: p.line)),
      ),
      child: Row(
        children: [
          Ico(kind == AttKind.image ? 'img' : 'file', color: p.accent),
          const SizedBox(width: 12),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  a.n,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: Fonts.sans(
                    size: 15,
                    w: FontWeight.w500,
                    color: p.text,
                    height: 1.3,
                  ),
                ),
                Mono(
                  '${a.t} · ${a.s}${multi ? ' · ${i + 1} / ${widget.list.length}' : ''}',
                  size: 10.5,
                  height: null,
                ),
              ],
            ),
          ),
          if (multi) ...[
            IBtn('arrowl', tip: t('Précédent (←)'), onTap: () => _go(-1)),
            Transform.flip(
              flipX: true,
              child: IBtn('arrowl', tip: t('Suivant (→)'), onTap: () => _go(1)),
            ),
            const SizedBox(width: 6),
          ],
          if (zoomable && !narrow) ...[
            IBtn('minus', tip: t('Zoom arrière (-)'), onTap: () => _zoom(-.2)),
            SizedBox(
              width: 48,
              child: Center(
                child: Mono(
                  '${(zoom * 100).round()} %',
                  size: 11,
                  ls: .04,
                  height: null,
                  color: p.muted,
                ),
              ),
            ),
            IBtn('plus', tip: t('Zoom avant (+)'), onTap: () => _zoom(.2)),
            const SizedBox(width: 10),
          ],
          Btn(
            icon: 'download',
            label: t('Télécharger'),
            iconOnly: narrow,
            onTap: () => s.download(a.n),
          ),
          const SizedBox(width: 6),
          IBtn(
            'x',
            tip: t('Fermer (Échap)'),
            onTap: () => Navigator.of(context).pop(),
          ),
        ],
      ),
    );

    Widget body = switch (kind) {
      AttKind.image => _Fetched(
        a,
        (x) => _ImageBody(x, tc, key: ValueKey('${x.n}$i')),
      ),
      AttKind.text => _Fetched(a, _TextBody.new),
      AttKind.pdf when realPdf => _Fetched(a, _PdfBody.new),
      AttKind.pdf => _PaperBody(documentPages(a), zoom),
      AttKind.other => _Unavailable(a, () => s.download(a.n)),
    };

    return SafeArea(
      child: Center(
        child: Padding(
          padding: EdgeInsets.all(narrow ? 8 : 24),
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 1100, maxHeight: 900),
            child: Focus(
              autofocus: true,
              onKeyEvent: (node, e) {
                if (e is! KeyDownEvent) return KeyEventResult.ignored;
                final k = e.logicalKey;
                if (k == LogicalKeyboardKey.arrowLeft) {
                  _go(-1);
                } else if (k == LogicalKeyboardKey.arrowRight) {
                  _go(1);
                } else if (e.character == '+' ||
                    k == LogicalKeyboardKey.numpadAdd) {
                  _zoom(.2);
                } else if (e.character == '-' ||
                    k == LogicalKeyboardKey.numpadSubtract) {
                  _zoom(-.2);
                } else {
                  return KeyEventResult.ignored;
                }
                return KeyEventResult.handled;
              },
              child: Material(
                color: p.bgSide,
                shape: RoundedRectangleBorder(side: BorderSide(color: p.line)),
                clipBehavior: Clip.antiAlias,
                child: Column(
                  children: [
                    header,
                    Expanded(
                      child: Container(
                        width: double.infinity,
                        color: p.bg,
                        child: AnimatedSwitcher(
                          duration: const Duration(milliseconds: 160),
                          child: KeyedSubtree(key: ValueKey(i), child: body),
                        ),
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}

// ----------------------------------------------------------- contenu réel

/// Affiche une pièce jointe du moteur une fois son contenu lu ; les autres passent directement.
class _Fetched extends StatefulWidget {
  final Attachment a;
  final Widget Function(Attachment loaded) builder;
  const _Fetched(this.a, this.builder);
  @override
  State<_Fetched> createState() => _FetchedState();
}

class _FetchedState extends State<_Fetched> {
  Future<Uint8List?>? _bytes;

  @override
  void initState() {
    super.initState();
    final a = widget.a;
    if (a.bytes == null && a.id != null) {
      _bytes = context.read<MailStore>().source?.attachmentBytes(a);
    }
  }

  @override
  Widget build(BuildContext context) {
    final a = widget.a;
    final pending = _bytes;
    if (pending == null) return widget.builder(a);
    final p = context.pal;
    return FutureBuilder<Uint8List?>(
      future: pending,
      builder: (ctx, snap) {
        if (snap.connectionState != ConnectionState.done) {
          return Center(
            child: Mono(t('Chargement…'), color: p.muted, height: null),
          );
        }
        final bytes = snap.data;
        if (bytes == null) {
          return _Unavailable(a, () => ctx.read<MailStore>().download(a.n));
        }
        return widget.builder(
          Attachment(a.n, a.t, a.s, bytes: bytes, path: a.path, id: a.id),
        );
      },
    );
  }
}

/// Document PDF réel, rendu page par page (défilement, zoom à la molette ou au pincement).
class _PdfBody extends StatelessWidget {
  final Attachment a;
  const _PdfBody(this.a);

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final params = PdfViewerParams(
      backgroundColor: p.bg,
      loadingBannerBuilder: (_, _, _) =>
          Center(child: Mono(t('Chargement…'), color: p.muted, height: null)),
      errorBannerBuilder: (_, _, _, _) => Center(
        child: Mono(t('Contenu indisponible.'), color: p.muted, height: null),
      ),
    );
    final bytes = a.bytes;
    return bytes != null
        ? PdfViewer.data(bytes, sourceName: a.id ?? a.n, params: params)
        : PdfViewer.file(a.path!, params: params);
  }
}

// ----------------------------------------------------------- images

class _ImageBody extends StatelessWidget {
  final Attachment a;
  final TransformationController tc;
  const _ImageBody(this.a, this.tc, {super.key});

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final Widget img = a.bytes != null
        ? Image.memory(
            a.bytes!,
            fit: BoxFit.contain,
            errorBuilder: (_, _, _) =>
                Center(child: Ico('img', size: 44, stroke: 1.1, color: p.dim)),
          )
        : AspectRatio(
            aspectRatio: 16 / 10,
            child: CustomPaint(
              painter: MockImagePainter(mockImageVariant(a), p.accent),
            ),
          );
    return InteractiveViewer(
      transformationController: tc,
      minScale: .5,
      maxScale: 6,
      boundaryMargin: const EdgeInsets.all(400),
      child: Center(
        child: Padding(padding: const EdgeInsets.all(24), child: img),
      ),
    );
  }
}

/// Maquettes fictives dessinées (pages d'accueil, logo) pour les images jointes de la démo.
class MockImagePainter extends CustomPainter {
  final int variant;
  final Color accent;
  MockImagePainter(this.variant, this.accent);

  @override
  void paint(Canvas c, Size s) {
    final w = s.width, h = s.height;
    final bg = Paint()..color = const Color(0xFF1b1b1b);
    final panel = Paint()..color = const Color(0xFF2a2a2a);
    final line = Paint()
      ..color = const Color(0xFF3a3a3a)
      ..style = PaintingStyle.stroke;
    final white = Paint()..color = const Color(0xFFecebe6);
    final gray = Paint()..color = const Color(0xFF75746f);
    final acc = Paint()..color = accent;
    void bar(Paint p, double x, double y, double bw, double bh) =>
        c.drawRect(Rect.fromLTWH(x * w, y * h, bw * w, bh * h), p);

    c.drawRect(Offset.zero & s, bg);
    c.drawRect(Offset.zero & s, line);

    if (variant == 9) {
      // logo : carré orange et losange
      final side = h * .42;
      final r = Rect.fromCenter(
        center: Offset(w / 2, h / 2),
        width: side,
        height: side,
      );
      c.drawRRect(RRect.fromRectAndRadius(r, Radius.circular(side * .06)), acc);
      final d = Path()
        ..moveTo(w / 2, h / 2 - side * .3)
        ..lineTo(w / 2 + side * .3, h / 2)
        ..lineTo(w / 2, h / 2 + side * .3)
        ..lineTo(w / 2 - side * .3, h / 2)
        ..close();
      c.drawPath(
        d,
        Paint()
          ..color = const Color(0xFF1b1b1b)
          ..style = PaintingStyle.stroke
          ..strokeWidth = side * .07
          ..strokeJoin = StrokeJoin.round,
      );
      return;
    }

    // en-tête commun
    bar(panel, 0, 0, 1, .09);
    bar(acc, .03, .025, .035, .045);
    bar(white, .08, .04, .09, .012);
    for (var k = 0; k < 3; k++) {
      bar(gray, .62 + k * .1, .043, .07, .01);
    }
    bar(acc, .9, .027, .07, .04);

    switch (variant) {
      case 0: // héros à gauche + téléphone
        bar(white, .06, .22, .36, .06);
        bar(white, .06, .31, .28, .06);
        bar(gray, .06, .44, .34, .018);
        bar(gray, .06, .48, .30, .018);
        bar(gray, .06, .52, .32, .018);
        bar(acc, .06, .62, .14, .07);
        c.drawRect(Rect.fromLTWH(.58 * w, .17 * h, .26 * w, .7 * h), panel);
        c.drawRect(Rect.fromLTWH(.58 * w, .17 * h, .26 * w, .7 * h), line);
        bar(gray, .6, .24, .16, .05);
        bar(acc, .67, .34, .15, .05);
        bar(gray, .6, .44, .12, .05);
        bar(acc, .67, .54, .15, .05);
      case 1: // centré + 3 cartes
        bar(white, .22, .2, .56, .07);
        bar(white, .3, .3, .4, .07);
        bar(gray, .32, .43, .36, .02);
        bar(acc, .43, .5, .14, .07);
        for (var k = 0; k < 3; k++) {
          final x = .08 + k * .30;
          bar(panel, x, .66, .26, .26);
          bar(acc, x + .02, .69, .04, .05);
          bar(white, x + .02, .78, .16, .02);
          bar(gray, x + .02, .83, .2, .015);
        }
      case 2: // tableau de bord
        bar(panel, 0, .09, .16, .91);
        for (var k = 0; k < 5; k++) {
          bar(k == 0 ? acc : gray, .02, .15 + k * .07, .1, .02);
        }
        bar(panel, .2, .16, .36, .34);
        bar(panel, .6, .16, .36, .34);
        for (var k = 0; k < 6; k++) {
          bar(acc, .23 + k * .05, .46 - (.05 + k * .035), .03, .05 + k * .035);
        }
        bar(white, .63, .2, .12, .03);
        bar(gray, .63, .27, .28, .015);
        bar(gray, .63, .31, .22, .015);
        bar(panel, .2, .56, .76, .34);
        for (var k = 0; k < 4; k++) {
          bar(k.isEven ? white : gray, .23, .61 + k * .07, .5, .018);
          bar(gray, .8, .61 + k * .07, .12, .018);
        }
      default: // image générique
        final stripe = Paint()..color = const Color(0xFF242424);
        for (var k = -10; k < 22; k += 2) {
          final path = Path()
            ..moveTo(k * w / 12, 0)
            ..lineTo((k + 1) * w / 12, 0)
            ..lineTo((k + 1) * w / 12 - h, h)
            ..lineTo(k * w / 12 - h, h)
            ..close();
          c.drawPath(path, stripe);
        }
        c.drawCircle(Offset(w * .5, h * .5), math.min(w, h) * .12, acc);
    }
  }

  @override
  bool shouldRepaint(MockImagePainter o) =>
      o.variant != variant || o.accent != accent;
}

// ----------------------------------------------------------- texte

class _TextBody extends StatelessWidget {
  final Attachment a;
  const _TextBody(this.a);

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final txt = a.bytes == null
        ? t('Contenu indisponible.')
        : utf8.decode(a.bytes!, allowMalformed: true);
    return ScrollArea(
      padding: const EdgeInsets.all(24),
      child: SelectableText(
        txt,
        style: Fonts.mono(size: 13, color: p.body, ls: 0, height: 1.6),
      ),
    );
  }
}

class _Unavailable extends StatelessWidget {
  final Attachment a;
  final VoidCallback onDownload;
  const _Unavailable(this.a, this.onDownload);

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Ico('file', size: 44, stroke: 1.1, color: p.dim),
          const SizedBox(height: 12),
          Mono(t('Aperçu indisponible')),
          const SizedBox(height: 8),
          Text(
            t('Ce type de fichier ne peut pas être lu ici.'),
            style: Fonts.sans(size: 14.5, color: p.dim),
          ),
          const SizedBox(height: 18),
          Btn(
            icon: 'download',
            label: t('Télécharger'),
            go: true,
            onTap: onDownload,
          ),
        ],
      ),
    );
  }
}

// ----------------------------------------------------------- papier (PDF fictif)

class _PaperBody extends StatelessWidget {
  final List<List<DocBlock>> pages;
  final double zoom;
  const _PaperBody(this.pages, this.zoom);

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, box) {
        final base = math.min(640.0, box.maxWidth - 32);
        final pw = base * zoom;
        final contentW = math.max(box.maxWidth, pw + 40);
        return ScrollArea(
          child: SingleChildScrollView(
            scrollDirection: Axis.horizontal,
            child: SizedBox(
              width: contentW,
              child: Padding(
                padding: const EdgeInsets.symmetric(vertical: 24),
                child: Column(
                  children: [
                    for (var n = 0; n < pages.length; n++)
                      Padding(
                        padding: EdgeInsets.only(
                          bottom: n == pages.length - 1 ? 0 : 24,
                        ),
                        child: _Page(
                          blocks: pages[n],
                          width: pw,
                          scale: pw / 640,
                          n: n + 1,
                          total: pages.length,
                        ),
                      ),
                  ],
                ),
              ),
            ),
          ),
        );
      },
    );
  }
}

class _Page extends StatelessWidget {
  final List<DocBlock> blocks;
  final double width, scale;
  final int n, total;
  const _Page({
    required this.blocks,
    required this.width,
    required this.scale,
    required this.n,
    required this.total,
  });

  static const ink = Color(0xFF1d1c1a);
  static const soft = Color(0xFF55534d);
  static const rule = Color(0xFFcdc9bd);

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final z = scale;
    TextStyle ts(
      double size, {
      FontWeight w = FontWeight.w400,
      Color c = ink,
      double h = 1.55,
    }) => Fonts.sans(size: size * z, w: w, color: c, height: h);

    Widget block(DocBlock b) {
      switch (b) {
        case DocTitle():
          return Padding(
            padding: EdgeInsets.only(bottom: 18 * z),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Container(
                  width: 34 * z,
                  height: 4 * z,
                  color: p.accent,
                  margin: EdgeInsets.only(bottom: 12 * z),
                ),
                Text(
                  b.title.toUpperCase(),
                  style: Fonts.display(size: 27 * z, color: ink, height: 1.1),
                ),
                if (b.sub != null)
                  Padding(
                    padding: EdgeInsets.only(top: 6 * z),
                    child: Text(
                      b.sub!.toUpperCase(),
                      style: Fonts.mono(
                        size: 9.5 * z,
                        color: soft,
                        ls: .12,
                        height: 1.4,
                      ),
                    ),
                  ),
              ],
            ),
          );
        case DocHeading():
          return Padding(
            padding: EdgeInsets.only(top: 12 * z, bottom: 6 * z),
            child: Text(b.text, style: ts(13, w: FontWeight.w700, h: 1.3)),
          );
        case DocPara():
          return Padding(
            padding: EdgeInsets.only(bottom: 10 * z),
            child: Text(b.text, style: ts(11.5)),
          );
        case DocBullets():
          return Padding(
            padding: EdgeInsets.only(bottom: 10 * z),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                for (final it in b.items)
                  Padding(
                    padding: EdgeInsets.only(bottom: 4 * z),
                    child: Row(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Container(
                          width: 5 * z,
                          height: 5 * z,
                          margin: EdgeInsets.only(top: 6 * z, right: 10 * z),
                          color: p.accent,
                        ),
                        Expanded(child: Text(it, style: ts(11.5))),
                      ],
                    ),
                  ),
              ],
            ),
          );
        case DocMeta():
          return Container(
            margin: EdgeInsets.only(bottom: 14 * z),
            padding: EdgeInsets.symmetric(vertical: 8 * z),
            decoration: const BoxDecoration(
              border: Border.symmetric(horizontal: BorderSide(color: rule)),
            ),
            child: Column(
              children: [
                for (final (k, v) in b.rows)
                  Padding(
                    padding: EdgeInsets.symmetric(vertical: 3 * z),
                    child: Row(
                      children: [
                        SizedBox(
                          width: 110 * z,
                          child: Text(
                            k.toUpperCase(),
                            style: Fonts.mono(
                              size: 9 * z,
                              color: soft,
                              ls: .1,
                              height: 1.4,
                            ),
                          ),
                        ),
                        Expanded(
                          child: Text(v, style: ts(11.5, w: FontWeight.w500)),
                        ),
                      ],
                    ),
                  ),
              ],
            ),
          );
        case DocTable():
          Widget cell(
            String s,
            int col, {
            bool head = false,
            bool bold = false,
          }) => Expanded(
            flex: col == 0 ? 5 : 2,
            child: Text(
              s,
              textAlign: col == 0 ? TextAlign.left : TextAlign.right,
              style: head
                  ? Fonts.mono(size: 9 * z, color: soft, ls: .08, height: 1.4)
                  : ts(11, w: bold ? FontWeight.w700 : FontWeight.w400),
            ),
          );
          return Padding(
            padding: EdgeInsets.only(bottom: 14 * z),
            child: Column(
              children: [
                Container(
                  padding: EdgeInsets.symmetric(vertical: 6 * z),
                  decoration: const BoxDecoration(
                    border: Border(bottom: BorderSide(color: ink)),
                  ),
                  child: Row(
                    children: [
                      for (var i = 0; i < b.head.length; i++)
                        cell(b.head[i].toUpperCase(), i, head: true),
                    ],
                  ),
                ),
                for (final r in b.rows)
                  Container(
                    padding: EdgeInsets.symmetric(vertical: 7 * z),
                    decoration: const BoxDecoration(
                      border: Border(bottom: BorderSide(color: rule)),
                    ),
                    child: Row(
                      children: [
                        for (var i = 0; i < r.length; i++) cell(r[i], i),
                      ],
                    ),
                  ),
                for (final (k, v) in b.totals)
                  Padding(
                    padding: EdgeInsets.only(top: 8 * z),
                    child: Row(
                      mainAxisAlignment: MainAxisAlignment.end,
                      children: [
                        Text(k, style: ts(11, c: soft)),
                        SizedBox(width: 22 * z),
                        SizedBox(
                          width: 90 * z,
                          child: Text(
                            v,
                            textAlign: TextAlign.right,
                            style: ts(11.5, w: FontWeight.w700),
                          ),
                        ),
                      ],
                    ),
                  ),
              ],
            ),
          );
        case DocSign():
          return Padding(
            padding: EdgeInsets.only(top: 26 * z),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Container(
                  width: 120 * z,
                  height: 1,
                  color: ink,
                  margin: EdgeInsets.only(bottom: 6 * z),
                ),
                Text(b.name, style: ts(11.5, w: FontWeight.w600, h: 1.3)),
                Text(
                  b.role.toUpperCase(),
                  style: Fonts.mono(
                    size: 9 * z,
                    color: soft,
                    ls: .1,
                    height: 1.4,
                  ),
                ),
              ],
            ),
          );
      }
    }

    return Container(
      width: width,
      height: width * 1.414,
      padding: EdgeInsets.fromLTRB(46 * z, 44 * z, 46 * z, 20 * z),
      decoration: const BoxDecoration(
        color: Color(0xFFfbfaf6),
        boxShadow: [
          BoxShadow(
            color: Color(0x66000000),
            blurRadius: 24,
            offset: Offset(0, 8),
          ),
        ],
      ),
      child: Column(
        children: [
          Expanded(
            child: ClipRect(
              child: SingleChildScrollView(
                physics: const NeverScrollableScrollPhysics(),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [for (final b in blocks) block(b)],
                ),
              ),
            ),
          ),
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(
                t('MIRA · DOCUMENT FICTIF'),
                style: Fonts.mono(
                  size: 8 * z,
                  color: soft,
                  ls: .1,
                  height: 1.4,
                ),
              ),
              Text(
                'PAGE $n / $total',
                style: Fonts.mono(
                  size: 8 * z,
                  color: soft,
                  ls: .1,
                  height: 1.4,
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}
