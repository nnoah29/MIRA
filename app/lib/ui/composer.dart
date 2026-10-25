import 'dart:math' as math;

import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:provider/provider.dart';

import '../data/format.dart';
import '../data/models.dart';
import '../state/rich_controller.dart';
import '../state/store.dart';
import '../state/template_insert.dart';
import '../theme/app_theme.dart';
import '../theme/icons.dart';
import 'composer_templates.dart';
import 'overlays.dart';
import '../data/documents.dart';
import 'viewer.dart';
import 'widgets.dart';
import '../i18n/i18n.dart';

Map<String, String> get _titles => {
  'new': t('Nouveau message'),
  'reply': t('Répondre'),
  'all': t('Répondre à tous'),
  'fwd': t('Transférer'),
  'draft': t('Brouillon'),
};

class Composer extends StatefulWidget {
  final ComposeState c;
  const Composer(this.c, {super.key});
  @override
  State<Composer> createState() => _ComposerState();
}

class _ComposerState extends State<Composer> {
  final scroll = ScrollController();
  final wrapKey = GlobalKey();
  double fieldW = 400;
  String _lastBody = '';

  ComposeState get c => widget.c;

  @override
  void initState() {
    super.initState();
    _lastBody = c.body.text;
    c.body.addListener(_onBody);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted) return;
      if (c.to.isNotEmpty) {
        c.bodyFocus.requestFocus();
        c.body.selection = const TextSelection.collapsed(offset: 0);
      } else {
        c.toFocus.requestFocus();
      }
    });
  }

  @override
  void didUpdateWidget(Composer old) {
    super.didUpdateWidget(old);
    if (old.c != widget.c) {
      old.c.body.removeListener(_onBody);
      _lastBody = widget.c.body.text;
      widget.c.body.addListener(_onBody);
    }
  }

  void _onBody() {
    final before = _lastBody;
    _lastBody = c.body.text;
    if (!mounted) return;
    setState(() {});
    if (slashJustTyped(before, _lastBody, c.body.selection.baseOffset)) {
      WidgetsBinding.instance.addPostFrameCallback(
        (_) => _templates(fromSlash: true),
      );
    }
  }

  /// Ouvre le sélecteur de modèles (bouton ou « / ») et insère le modèle choisi.
  void _templates({bool fromSlash = false}) {
    final s = context.read<MailStore>();
    final admin = s.source?.templateAdmin;
    if (!mounted || admin == null) return;
    final sender = s.senderChoices().where(
      (x) => x.email.toLowerCase() == c.from.toLowerCase(),
    );
    pickAndInsertTemplate(
      context,
      c: c,
      admin: admin,
      fromName: sender.firstOrNull?.name,
      fromSlash: fromSlash,
      notify: s.toast,
      touch: s.touch,
    );
  }

  @override
  void dispose() {
    c.body.removeListener(_onBody);
    scroll.dispose();
    super.dispose();
  }

  Future<void> _attach() async {
    final s = context.read<MailStore>();
    try {
      final files = await FilePicker.pickFiles();
      for (final f in files) {
        final len = await f.length() ?? 0;
        final name = f.name;
        final ext = (f.extension ?? 'FILE');
        final att = Attachment(
          name,
          (ext.length > 4 ? ext.substring(0, 4) : ext).toUpperCase(),
          sizeOf(len),
        );
        final readable = isImageAtt(att) || isTextAtt(att);
        c.atts.add(
          Attachment(
            att.n,
            att.t,
            att.s,
            bytes: readable && len <= 8 * 1024 * 1024
                ? await f.readAsBytes()
                : null,
            path: f.path,
          ),
        );
      }
      s.touch();
    } catch (_) {
      s.toast(t('Impossible d’ouvrir le sélecteur de fichiers'), err: true);
    }
  }

  Future<void> _link() async {
    final p = context.pal;
    final ctl = TextEditingController(text: 'https://');
    final ok = await showDialog<bool>(
      context: context,
      builder: (ctx) => Dialog(
        backgroundColor: p.bgSide,
        shape: RoundedRectangleBorder(side: BorderSide(color: p.line)),
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 420),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  t('Adresse du lien'),
                  style: Fonts.sans(size: 15, color: p.text),
                ),
                const SizedBox(height: 14),
                Container(
                  padding: const EdgeInsets.symmetric(
                    horizontal: 12,
                    vertical: 9,
                  ),
                  decoration: BoxDecoration(
                    color: p.surface,
                    border: Border.all(color: p.line),
                  ),
                  child: TextField(
                    controller: ctl,
                    autofocus: true,
                    onSubmitted: (_) => Navigator.pop(ctx, true),
                    style: Fonts.sans(size: 14.5, color: p.text),
                    cursorColor: p.accent,
                    decoration: const InputDecoration(
                      isDense: true,
                      border: InputBorder.none,
                      contentPadding: EdgeInsets.zero,
                    ),
                  ),
                ),
                const SizedBox(height: 18),
                Row(
                  mainAxisAlignment: MainAxisAlignment.end,
                  children: [
                    Btn(
                      label: t('Annuler'),
                      onTap: () => Navigator.pop(ctx, false),
                    ),
                    const SizedBox(width: 10),
                    Btn(
                      label: t('OK'),
                      go: true,
                      onTap: () => Navigator.pop(ctx, true),
                    ),
                  ],
                ),
              ],
            ),
          ),
        ),
      ),
    );
    if (!mounted) return;
    if (ok == true && ctl.text.trim().isNotEmpty) {
      if (!c.body.isOn(Fx.link)) c.body.toggle(Fx.link);
      context.read<MailStore>().touch();
    }
    c.bodyFocus.requestFocus();
  }

  void _cmd(String k) {
    final s = context.read<MailStore>();
    switch (k) {
      case 'b':
        c.body.toggle(Fx.bold);
      case 'i':
        c.body.toggle(Fx.italic);
      case 'u':
        c.body.toggle(Fx.underline);
      case 'left':
        c.center = false;
      case 'center':
        c.center = true;
      case 'list':
        c.body.toggleBullets();
      case 'link':
        _link();
        return;
    }
    c.bodyFocus.requestFocus();
    s.touch();
  }

  @override
  Widget build(BuildContext context) {
    final s = context.watch<MailStore>();
    final p = context.pal;
    final w = MediaQuery.sizeOf(context).width;
    final narrow = w <= 1100;
    final hp = narrow ? 16.0 : 32.0;
    final mobile = w <= 760;
    final bodyStyle = Fonts.sans(size: 16, color: p.body, height: 1.8);
    c.body
      ..baseColor = p.body
      ..strongColor = p.text
      ..accent = p.accent
      ..codeBg = p.surface;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Padding(
          padding: EdgeInsets.fromLTRB(hp, 24, hp, 12),
          child: Row(
            children: [
              Text(
                (_titles[c.mode] ?? t('Nouveau message')).toUpperCase(),
                style: Fonts.display(size: 24, ls: .02, color: p.text),
              ),
              const Spacer(),
              Mono(c.saved, height: null),
              const SizedBox(width: 12),
              IBtn(
                'x',
                tip: t('Fermer (Échap)'),
                onTap: () => s.closeCompose(true),
              ),
            ],
          ),
        ),
        Padding(
          padding: EdgeInsets.symmetric(horizontal: hp),
          child: Column(
            children: [
              _Field(
                label: t('De'),
                child: Builder(
                  builder: (ctx) => Hov(
                    onTap: () => Pops.show(ctx, 'from', (pc, close) {
                      return Column(
                        crossAxisAlignment: CrossAxisAlignment.stretch,
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          for (final e in s.senderChoices())
                            PopItem(
                              leading: const SizedBox.shrink(),
                              label: e.name.isEmpty || e.name == e.email
                                  ? e.email
                                  : '${e.name} <${e.email}>',
                              on: e.email == c.from,
                              onTap: () {
                                close();
                                s.changeFrom(e.email);
                              },
                            ),
                        ],
                      );
                    }),
                    builder: (_, _) => Row(
                      children: [
                        Expanded(
                          child: Text(
                            c.from,
                            style: Fonts.sans(size: 14.5, color: p.text),
                          ),
                        ),
                        Ico('chev', size: 14, color: p.muted),
                      ],
                    ),
                  ),
                ),
              ),
              _Shake(
                n: c.shake,
                child: _Field(
                  label: t('À'),
                  child: RecipientBox(
                    list: c.to,
                    onChange: s.touch,
                    focus: c.toFocus,
                  ),
                ),
              ),
              _Field(
                label: t('Cc'),
                trailing: Hov(
                  onTap: () {
                    c.showBcc = !c.showBcc;
                    s.refresh();
                  },
                  tip: t('Afficher Cci'),
                  builder: (_, h) => Container(
                    width: 38,
                    height: 38,
                    margin: const EdgeInsets.only(left: 0),
                    decoration: BoxDecoration(
                      border: Border.all(color: p.line),
                    ),
                    alignment: Alignment.center,
                    child: Ico('chev', color: p.muted),
                  ),
                ),
                child: RecipientBox(list: c.cc, onChange: s.touch),
              ),
              if (c.showBcc)
                _Field(
                  label: t('Cci'),
                  child: RecipientBox(list: c.bcc, onChange: s.touch),
                ),
            ],
          ),
        ),
        Container(
          margin: EdgeInsets.fromLTRB(hp, 10, hp, 0),
          padding: const EdgeInsets.symmetric(vertical: 16),
          decoration: BoxDecoration(
            border: Border.symmetric(horizontal: BorderSide(color: p.lineSoft)),
          ),
          child: TextField(
            controller: c.subject,
            onChanged: (_) => s.touch(),
            cursorColor: p.accent,
            cursorWidth: 1,
            style: Fonts.sans(size: 15, w: FontWeight.w500, color: p.text),
            decoration: InputDecoration(
              isDense: true,
              border: InputBorder.none,
              contentPadding: EdgeInsets.zero,
              hintText: t('Objet'),
              hintStyle: Fonts.sans(size: 15, w: FontWeight.w500, color: p.dim),
            ),
          ),
        ),
        Expanded(
          child: LayoutBuilder(
            builder: (ctx, box) {
              final textW = box.maxWidth - 2 * hp;
              final sel = c.body.selection;
              Widget? fmt;
              if (sel.isValid &&
                  !sel.isCollapsed &&
                  c.bodyFocus.hasFocus &&
                  sel.end <= c.body.text.length) {
                final tp = TextPainter(
                  text: c.body.buildTextSpan(
                    context: ctx,
                    style: bodyStyle,
                    withComposing: false,
                  ),
                  textDirection: TextDirection.ltr,
                  textAlign: c.center ? TextAlign.center : TextAlign.start,
                )..layout(maxWidth: textW);
                final boxes = tp.getBoxesForSelection(sel);
                if (boxes.isNotEmpty) {
                  final top = boxes.map((b) => b.top).reduce(math.min);
                  final bottom = boxes.map((b) => b.bottom).reduce(math.max);
                  final left = boxes.map((b) => b.left).reduce(math.min);
                  final off = scroll.hasClients ? scroll.offset : 0.0;
                  const fw = 232.0, fh = 40.0;
                  final x = (left + hp - 0)
                      .clamp(8.0, math.max(8.0, box.maxWidth - fw - 8))
                      .toDouble();
                  final above = top + 24 - off - fh - 8;
                  final y = above >= 4 ? above : bottom + 24 - off + 8;
                  fmt = Positioned(
                    left: x,
                    top: y,
                    child: _FmtBar(c: c, onCmd: _cmd),
                  );
                }
              }
              return Stack(
                children: [
                  Positioned.fill(
                    child: Scrollbar(
                      controller: scroll,
                      child: SingleChildScrollView(
                        controller: scroll,
                        padding: EdgeInsets.fromLTRB(hp, 24, hp, 24),
                        child: ConstrainedBox(
                          constraints: BoxConstraints(
                            minHeight: math.max(0, box.maxHeight - 48),
                          ),
                          child: IntrinsicHeight(
                            child: Column(
                              crossAxisAlignment: CrossAxisAlignment.stretch,
                              children: [
                                TextField(
                                  controller: c.body,
                                  strutStyle: StrutStyle.disabled,
                                  focusNode: c.bodyFocus,
                                  maxLines: null,
                                  onChanged: (_) => s.touch(),
                                  textAlign: c.center
                                      ? TextAlign.center
                                      : TextAlign.start,
                                  cursorColor: p.accent,
                                  cursorWidth: 1,
                                  style: bodyStyle,
                                  decoration: InputDecoration(
                                    isDense: true,
                                    border: InputBorder.none,
                                    contentPadding: EdgeInsets.zero,
                                    hintText: t('Écris ton message…'),
                                    hintStyle: bodyStyle.copyWith(color: p.dim),
                                  ),
                                ),
                                if (c.quote != null)
                                  blocksView(
                                    c.quote!,
                                    bodyStyle,
                                    p,
                                    gap: 16,
                                    maxW: 700,
                                  ),
                                Expanded(
                                  child: GestureDetector(
                                    behavior: HitTestBehavior.opaque,
                                    onTap: () {
                                      c.bodyFocus.requestFocus();
                                      c.body.selection =
                                          TextSelection.collapsed(
                                            offset: c.body.text.length,
                                          );
                                    },
                                    child: const SizedBox(
                                      width: double.infinity,
                                      height: 24,
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
                  ?fmt,
                ],
              );
            },
          ),
        ),
        if (c.atts.isNotEmpty)
          Padding(
            padding: EdgeInsets.fromLTRB(hp, 0, hp, 18),
            child: Align(
              alignment: Alignment.centerLeft,
              child: Wrap(
                spacing: 14,
                runSpacing: 14,
                children: [
                  for (var i = 0; i < c.atts.length; i++)
                    _Thumb(
                      c.atts[i],
                      onOpen: () => showAttachmentViewer(context, c.atts, i),
                      onRemove: () {
                        c.atts.removeAt(i);
                        s.touch();
                      },
                    ),
                ],
              ),
            ),
          ),
        Container(
          padding: EdgeInsets.symmetric(horizontal: hp, vertical: 14),
          decoration: BoxDecoration(
            border: Border(top: BorderSide(color: p.lineSoft)),
          ),
          child: Row(
            children: [
              IBtn('clip', tip: t('Joindre un fichier'), onTap: _attach),
              if (s.source?.templateAdmin != null) ...[
                const SizedBox(width: 10),
                IBtn(
                  'file',
                  key: const ValueKey('insert-template'),
                  tip: t('Insérer un modèle'),
                  onTap: _templates,
                ),
              ],
              const SizedBox(width: 10),
              IBtn('trash', tip: t('Supprimer le brouillon'), onTap: s.discard),
              const Spacer(),
              Builder(
                builder: (ctx) => Btn(
                  icon: 'clock',
                  label: t('Programmer'),
                  iconOnly: mobile && false,
                  onTap: () => showSchedPop(ctx),
                ),
              ),
              const SizedBox(width: 10),
              Btn(
                icon: 'send',
                label: t('Envoyer'),
                go: true,
                tip: t('Envoyer (Ctrl+Entrée)'),
                onTap: () => s.send(),
              ),
            ],
          ),
        ),
      ],
    );
  }
}

class _Field extends StatelessWidget {
  final String label;
  final Widget child;
  final Widget? trailing;
  const _Field({required this.label, required this.child, this.trailing});
  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Padding(
      padding: const EdgeInsets.only(bottom: 10),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          SizedBox(width: 48, child: Mono(label, height: null)),
          const SizedBox(width: 16),
          Expanded(
            child: Container(
              constraints: const BoxConstraints(minHeight: 38),
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 5),
              decoration: BoxDecoration(
                color: p.surface,
                border: Border.all(color: p.line),
              ),
              alignment: Alignment.centerLeft,
              child: child,
            ),
          ),
          if (trailing != null) ...[const SizedBox(width: 16), trailing!],
        ],
      ),
    );
  }
}

class _Shake extends StatelessWidget {
  final int n;
  final Widget child;
  const _Shake({required this.n, required this.child});
  @override
  Widget build(BuildContext context) {
    if (n == 0) return child;
    return TweenAnimationBuilder<double>(
      key: ValueKey(n),
      tween: Tween(begin: 0, end: 1),
      duration: const Duration(milliseconds: 300),
      builder: (_, t, c) {
        final dx = t < .25
            ? -5 * (t / .25)
            : (t < .75
                  ? -5 + 10 * ((t - .25) / .5)
                  : 5 * (1 - (t - .75) / .25));
        return Transform.translate(offset: Offset(dx, 0), child: c);
      },
      child: child,
    );
  }
}

class _FmtBar extends StatelessWidget {
  final ComposeState c;
  final void Function(String) onCmd;
  const _FmtBar({required this.c, required this.onCmd});

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    Widget b(
      String k,
      Widget Function(Color) icon, {
      bool on = false,
      String? tip,
    }) => Hov(
      onTap: () => onCmd(k),
      tip: tip,
      builder: (_, h) => Container(
        width: 28,
        height: 28,
        alignment: Alignment.center,
        color: on ? p.accentSoft : (h ? p.surface : null),
        child: icon(on ? p.accent : (h ? p.text : p.muted)),
      ),
    );
    Widget letter(
      String t,
      Color col, {
      FontStyle? style,
      TextDecoration? deco,
    }) => Text(
      t,
      style: Fonts.mono(
        size: 13,
        w: FontWeight.w500,
        color: col,
        ls: 0,
        height: 1.2,
      ).copyWith(fontStyle: style, decoration: deco),
    );
    Widget hr() => Container(
      width: 1,
      height: 18,
      margin: const EdgeInsets.symmetric(horizontal: 4),
      color: p.line,
    );
    final bd = c.body;
    return Container(
      padding: const EdgeInsets.all(5),
      decoration: BoxDecoration(
        color: p.surface2,
        border: Border.all(color: p.line),
        boxShadow: const [
          BoxShadow(
            color: Color(0x66000000),
            blurRadius: 24,
            offset: Offset(0, 8),
          ),
        ],
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          b(
            'b',
            (col) => letter('B', col),
            on: bd.isOn(Fx.bold | Fx.strong) || bd.isOn(Fx.bold),
            tip: t('Gras'),
          ),
          const SizedBox(width: 2),
          b(
            'i',
            (col) => letter('I', col, style: FontStyle.italic),
            on: bd.isOn(Fx.italic),
            tip: t('Italique'),
          ),
          const SizedBox(width: 2),
          b(
            'u',
            (col) => letter('U', col, deco: TextDecoration.underline),
            on: bd.isOn(Fx.underline),
            tip: t('Souligné'),
          ),
          hr(),
          b(
            'left',
            (col) => Ico('alignl', size: 14, color: col),
            on: !c.center,
            tip: t('Aligner à gauche'),
          ),
          const SizedBox(width: 2),
          b(
            'center',
            (col) => Ico('alignc', size: 14, color: col),
            on: c.center,
            tip: t('Centrer'),
          ),
          hr(),
          b('link', (col) => Ico('link', size: 14, color: col), tip: t('Lien')),
          const SizedBox(width: 2),
          b(
            'list',
            (col) => Ico('list', size: 14, color: col),
            tip: t('Liste'),
          ),
        ],
      ),
    );
  }
}

class _Thumb extends StatelessWidget {
  final Attachment a;
  final VoidCallback onRemove, onOpen;
  const _Thumb(this.a, {required this.onRemove, required this.onOpen});
  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final isImg = a.bytes != null && isImageAtt(a);
    return SizedBox(
      width: 78,
      child: Stack(
        clipBehavior: Clip.none,
        children: [
          Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Hov(
                tip: t('Lire le fichier'),
                onTap: onOpen,
                builder: (_, _) => Container(
                  width: 78,
                  height: 78,
                  decoration: BoxDecoration(
                    color: p.surface,
                    border: Border.all(color: p.line),
                  ),
                  child: isImg
                      ? Image.memory(
                          a.bytes!,
                          fit: BoxFit.cover,
                          errorBuilder: (_, _, _) => const SizedBox(),
                        )
                      : Stack(
                          children: [
                            Center(child: Ico('file', color: p.dim)),
                            Positioned(
                              left: 6,
                              bottom: 6,
                              child: Container(
                                padding: const EdgeInsets.symmetric(
                                  horizontal: 5,
                                  vertical: 2,
                                ),
                                color: p.accent,
                                child: Text(
                                  a.t,
                                  style: Fonts.mono(
                                    size: 10.5,
                                    w: FontWeight.w700,
                                    color: p.onAccent,
                                    ls: .1,
                                    height: 1.2,
                                  ),
                                ),
                              ),
                            ),
                          ],
                        ),
                ),
              ),
              const SizedBox(height: 6),
              Text(
                a.n,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: Fonts.sans(size: 12.5, color: p.text, height: 1.3),
              ),
              Text(
                a.s,
                style: Fonts.sans(size: 11.5, color: p.dim, height: 1.5),
              ),
            ],
          ),
          Positioned(
            right: -6,
            top: -6,
            child: Hov(
              onTap: onRemove,
              builder: (_, _) => Container(
                width: 16,
                height: 16,
                color: p.text,
                alignment: Alignment.center,
                child: Ico(
                  'x',
                  size: 9,
                  stroke: 3,
                  color: const Color(0xFF1b1b1b),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

// ============================ Destinataires ============================

final _emailRe = RegExp(r'[^\s<>,;]+@[^\s<>,;]+\.[^\s<>,;]+');

class RecipientBox extends StatefulWidget {
  final List<Person> list;
  final VoidCallback onChange;
  final FocusNode? focus;
  const RecipientBox({
    super.key,
    required this.list,
    required this.onChange,
    this.focus,
  });
  @override
  State<RecipientBox> createState() => _RecipientBoxState();
}

class _RecipientBoxState extends State<RecipientBox> {
  final ctl = TextEditingController();
  late final FocusNode fn = widget.focus ?? FocusNode();
  bool bad = false;

  void _onBlur() {
    if (!fn.hasFocus && ctl.text.trim().isNotEmpty) _add(ctl.text);
  }

  @override
  void initState() {
    super.initState();
    fn.addListener(_onBlur);
  }

  @override
  void dispose() {
    fn.removeListener(_onBlur);
    super.dispose();
  }

  bool _add(String v, {bool refocus = false}) {
    final cleaned = v.trim().replaceAll(RegExp(r'[,;]+$'), '');
    final m = _emailRe.firstMatch(cleaned);
    if (m == null) {
      setState(() => bad = true);
      return false;
    }
    final em = m.group(0)!;
    Person? known;
    for (final c in context.read<MailStore>().contactPeople()) {
      if (c.email.toLowerCase() == em.toLowerCase()) known = c;
    }
    if (!widget.list.any((p) => p.email == em)) {
      widget.list.add(known ?? Person(em, em));
    }
    ctl.clear();
    setState(() => bad = false);
    widget.onChange();
    if (refocus) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) fn.requestFocus();
      });
    }
    return true;
  }

  List<Person> _options(String t) {
    final q = t.trim().toLowerCase();
    if (q.isEmpty) return const [];
    bool starts(Person c) =>
        c.email.toLowerCase().startsWith(q) ||
        c.name.toLowerCase().split(' ').any((w) => w.startsWith(q));
    final hits =
        context
            .read<MailStore>()
            .contactPeople()
            .where(
              (c) =>
                  c.email.toLowerCase().contains(q) ||
                  c.name.toLowerCase().contains(q),
            )
            .toList()
          ..sort((a, b) => (starts(b) ? 1 : 0) - (starts(a) ? 1 : 0));
    return hits.take(6).toList();
  }

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return LayoutBuilder(
      builder: (context, box) => Stack(
        alignment: Alignment.centerRight,
        children: [
          GestureDetector(
            behavior: HitTestBehavior.opaque,
            onTap: fn.requestFocus,
            child: Wrap(
              spacing: 8,
              runSpacing: 4,
              crossAxisAlignment: WrapCrossAlignment.center,
              children: [
                for (var i = 0; i < widget.list.length; i++)
                  Tooltip(
                    message: widget.list[i].email,
                    child: Container(
                      padding: const EdgeInsets.symmetric(
                        horizontal: 8,
                        vertical: 3,
                      ),
                      decoration: BoxDecoration(
                        color: p.surface2,
                        border: Border.all(color: p.line),
                      ),
                      child: Row(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          Text(
                            widget.list[i].name == widget.list[i].email
                                ? widget.list[i].email
                                : widget.list[i].name,
                            style: Fonts.sans(
                              size: 13.5,
                              color: p.text,
                              height: 1.4,
                            ),
                          ),
                          const SizedBox(width: 8),
                          Hov(
                            onTap: () {
                              widget.list.removeAt(i);
                              setState(() {});
                              widget.onChange();
                            },
                            builder: (_, h) =>
                                Ico('x', size: 10, color: h ? p.accent : p.dim),
                          ),
                        ],
                      ),
                    ),
                  ),
                SizedBox(
                  width: widget.list.isEmpty
                      ? box.maxWidth
                      : math.min(box.maxWidth, 340.0),
                  child: RawAutocomplete<Person>(
                    textEditingController: ctl,
                    focusNode: fn,
                    displayStringForOption: (o) => o.email,
                    optionsBuilder: (v) => _options(v.text),
                    onSelected: (o) =>
                        WidgetsBinding.instance.addPostFrameCallback(
                          (_) => _add(o.email, refocus: true),
                        ),
                    fieldViewBuilder: (ctx, tc, f, submit) => Focus(
                      onKeyEvent: (node, e) {
                        if (e is! KeyDownEvent) return KeyEventResult.ignored;
                        final t = tc.text;
                        if (e.logicalKey == LogicalKeyboardKey.backspace &&
                            t.isEmpty &&
                            widget.list.isNotEmpty) {
                          widget.list.removeLast();
                          setState(() {});
                          widget.onChange();
                          return KeyEventResult.handled;
                        }
                        final ch = e.character;
                        final isSep = ch == ',' || ch == ';' || ch == ' ';
                        final isEnter =
                            e.logicalKey == LogicalKeyboardKey.enter ||
                            e.logicalKey == LogicalKeyboardKey.numpadEnter;
                        final isTab = e.logicalKey == LogicalKeyboardKey.tab;
                        if ((isSep || isEnter || isTab) &&
                            t.trim().isNotEmpty) {
                          if (isEnter &&
                              !_emailRe.hasMatch(t) &&
                              _options(t).isNotEmpty) {
                            return KeyEventResult
                                .ignored; // laisse l'autocomplétion choisir
                          }
                          final ok = _add(t);
                          if (!ok || !isTab) return KeyEventResult.handled;
                        }
                        return KeyEventResult.ignored;
                      },
                      child: TextField(
                        controller: tc,
                        focusNode: f,
                        onSubmitted: (_) => submit(),
                        onEditingComplete: () {}, // garde le focus après Entrée
                        cursorColor: p.accent,
                        cursorWidth: 1,
                        onChanged: (_) {
                          if (bad) setState(() => bad = false);
                        },
                        style: Fonts.sans(
                          size: 14.5,
                          color: bad ? p.accent : p.text,
                        ),
                        decoration: InputDecoration(
                          isDense: true,
                          border: InputBorder.none,
                          contentPadding: const EdgeInsets.symmetric(
                            vertical: 6,
                          ),
                          hintText: widget.focus != null && widget.list.isEmpty
                              ? t('Ajouter un destinataire…')
                              : null,
                          hintStyle: Fonts.sans(size: 14.5, color: p.muted),
                        ),
                      ),
                    ),
                    optionsViewBuilder: (ctx, onSel, opts) => Align(
                      alignment: Alignment.topLeft,
                      child: Material(
                        color: p.surface2,
                        elevation: 0,
                        child: Container(
                          width: 340,
                          constraints: const BoxConstraints(maxHeight: 320),
                          decoration: BoxDecoration(
                            color: p.surface2,
                            border: Border.all(color: p.line),
                            boxShadow: const [
                              BoxShadow(
                                color: Color(0x66000000),
                                blurRadius: 36,
                                offset: Offset(0, 14),
                              ),
                            ],
                          ),
                          child: ListView(
                            padding: const EdgeInsets.all(6),
                            shrinkWrap: true,
                            children: [
                              for (final o in opts)
                                Hov(
                                  onTap: () => onSel(o),
                                  builder: (_, h) => Container(
                                    color:
                                        h ||
                                            AutocompleteHighlightedOption.of(
                                                  ctx,
                                                ) ==
                                                opts.toList().indexOf(o)
                                        ? p.surface
                                        : null,
                                    padding: const EdgeInsets.symmetric(
                                      horizontal: 12,
                                      vertical: 8,
                                    ),
                                    child: Column(
                                      crossAxisAlignment:
                                          CrossAxisAlignment.start,
                                      children: [
                                        Text(
                                          o.email,
                                          style: Fonts.sans(
                                            size: 14,
                                            color: p.text,
                                          ),
                                        ),
                                        Text(
                                          o.name,
                                          style: Fonts.sans(
                                            size: 12,
                                            color: p.dim,
                                          ),
                                        ),
                                      ],
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
              ],
            ),
          ),
        ],
      ),
    );
  }
}
