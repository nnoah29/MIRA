import 'dart:async';

import 'body_skeleton.dart';
import 'html_body.dart';

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../data/format.dart';
import '../data/models.dart';
import '../state/store.dart';
import '../theme/app_theme.dart';
import '../theme/icons.dart';
import 'viewer.dart';
import 'original_dialog.dart';
import 'widgets.dart';
import '../i18n/i18n.dart';

class Reader extends StatefulWidget {
  final Mail mail;
  const Reader(this.mail, {super.key});
  @override
  State<Reader> createState() => _ReaderState();
}

class _ReaderState extends State<Reader> {
  final List<String> deck = [];
  Timer? _warm;

  /// Garde les dernières conversations montées hors écran : y revenir est instantané.
  void _track() {
    deck.remove(widget.mail.id);
    deck.add(widget.mail.id);
    while (deck.length > _deckSize) {
      deck.removeAt(0);
    }
  }

  /// Monte hors écran, une à la fois et au calme, les voisins de la conversation affichée.
  void _scheduleWarmUp(MailStore s) {
    _warm?.cancel();
    if (!widget.mail.msgs.every((x) => x.loaded || x.ref == null)) return;
    _warm = Timer(const Duration(milliseconds: 450), () {
      final l = s.visible();
      final i = l.indexWhere((x) => x.id == widget.mail.id);
      final next = [i + 1, i - 1]
          .where((j) => i >= 0 && j >= 0 && j < l.length)
          .map((j) => l[j])
          .where((x) => !deck.contains(x.id));
      if (next.isEmpty || !mounted) return;
      setState(() {
        if (deck.length >= _deckSize) deck.removeAt(0);
        deck.insert(0, next.first.id);
      });
    });
  }

  @override
  void dispose() {
    _warm?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    _track();
    final s = context.watch<MailStore>();
    _scheduleWarmUp(s);
    final p = context.pal;
    final m = widget.mail;
    final w = MediaQuery.sizeOf(context).width;
    final narrow = w <= 1100, mobile = w <= 760;
    final hp = narrow ? 16.0 : 32.0;
    // En CSS, les boutons icône rétrécissent (jusqu'à 18 px) quand la barre manque de place.
    final shrinkW = (38 - ((391 - (w - 2 * hp)).clamp(0, 1000)) / 2)
        .clamp(18, 38)
        .toDouble();

    return Column(
      children: [
        if (m.outbox != null)
          _OutboxBar(m, hp)
        else
          Container(
            padding: EdgeInsets.symmetric(horizontal: hp, vertical: 14),
            decoration: BoxDecoration(
              border: Border(bottom: BorderSide(color: p.lineSoft)),
            ),
            child: LayoutBuilder(
              builder: (context, box) => SingleChildScrollView(
                scrollDirection: Axis.horizontal,
                physics: const NeverScrollableScrollPhysics(),
                child: ConstrainedBox(
                  constraints: BoxConstraints(minWidth: box.maxWidth),
                  child: IntrinsicWidth(
                    child: Row(
                      children: [
                        Btn(
                          icon: 'reply',
                          label: t('Répondre'),
                          go: true,
                          padding: mobile
                              ? const EdgeInsets.symmetric(horizontal: 14)
                              : null,
                          tip: t('Répondre (r)'),
                          onTap: () => s.replyTo('reply'),
                        ),
                        const SizedBox(width: 8),
                        Btn(
                          icon: 'reply',
                          label: t('Tous'),
                          iconOnly: mobile,
                          width: shrinkW,
                          padding: mobile ? EdgeInsets.zero : null,
                          tip: t('Répondre à tous (a)'),
                          onTap: () => s.replyTo('all'),
                        ),
                        const SizedBox(width: 8),
                        Btn(
                          icon: 'fwd',
                          label: t('Transférer'),
                          iconOnly: mobile,
                          width: shrinkW,
                          padding: mobile ? EdgeInsets.zero : null,
                          tip: t('Transférer (f)'),
                          onTap: () => s.replyTo('fwd'),
                        ),
                        const Spacer(),
                        IBtn(
                          'star',
                          on: m.starred,
                          filled: m.starred,
                          tip: t('Favori (s)'),
                          onTap: () => s.toggleStar(),
                        ),
                        const SizedBox(width: 8),
                        IBtn(
                          'archive',
                          tip: m.folder == 'archive'
                              ? t('Désarchiver')
                              : t('Archiver (e)'),
                          onTap: s.archive,
                        ),
                        const SizedBox(width: 8),
                        if (s.unsubFor(m) != null) ...[
                          Btn(
                            label: s.isUnsubscribed(m)
                                ? t('Désabonné')
                                : t('Se désabonner'),
                            tip: t('Se désabonner'),
                            onTap: s.isUnsubscribed(m)
                                ? null
                                : () => _confirmUnsubscribe(context, m),
                          ),
                          const SizedBox(width: 8),
                        ],
                        IBtn(
                          'warn',
                          tip: m.folder == 'spam'
                              ? t('Pas du spam')
                              : t('Spam'),
                          onTap: s.spam,
                        ),
                        const SizedBox(width: 8),
                        IBtn('trash', tip: t('Supprimer (#)'), onTap: s.trash),
                        if (s.source != null) ...[
                          const SizedBox(width: 8),
                          IBtn(
                            'file',
                            key: const ValueKey('show-original'),
                            tip: t('Afficher l’original'),
                            onTap: () => showOriginal(context, m),
                          ),
                        ],
                      ],
                    ),
                  ),
                ),
              ),
            ),
          ),
        Expanded(
          child: _Deck(current: m, ids: deck),
        ),
      ],
    );
  }
}

/// Barre d'un envoi de la boîte d'envoi : où il en est, et ce qu'on peut en faire.
class _OutboxBar extends StatelessWidget {
  final Mail mail;
  final double hp;
  const _OutboxBar(this.mail, this.hp);

  @override
  Widget build(BuildContext context) {
    final s = context.read<MailStore>();
    final p = context.pal;
    final o = mail.outbox!;
    final text = switch (o.state) {
      'failed' => t('L’envoi a échoué : {0}', [
        o.error ?? t('erreur inconnue'),
      ]),
      'uncertain' => t(
        'Le résultat de l’envoi est incertain : le message est peut-être parti.',
      ),
      _ => t('Départ prévu à {0}.', [hhmm(o.at)]),
    };
    return Container(
      padding: EdgeInsets.symmetric(horizontal: hp, vertical: 14),
      decoration: BoxDecoration(
        border: Border(bottom: BorderSide(color: p.lineSoft)),
      ),
      child: Wrap(
        spacing: 10,
        runSpacing: 8,
        crossAxisAlignment: WrapCrossAlignment.center,
        children: [
          ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 520),
            child: Text(text, style: Fonts.sans(size: 14, color: p.text)),
          ),
          if (o.waiting)
            Btn(
              key: const ValueKey('outbox-cancel'),
              icon: 'x',
              label: t('Annuler l’envoi'),
              onTap: () => s.cancelOutbox(mail),
            ),
          if (o.state == 'failed')
            Btn(
              key: const ValueKey('outbox-retry'),
              icon: 'refresh',
              label: t('Réessayer'),
              go: true,
              onTap: () => s.retryOutbox(mail),
            ),
          if (o.state == 'uncertain') ...[
            Btn(
              key: const ValueKey('outbox-resend'),
              icon: 'send',
              label: t('Renvoyer'),
              go: true,
              onTap: () => s.resolveOutbox(mail, resend: true),
            ),
            Btn(
              key: const ValueKey('outbox-sent'),
              icon: 'check',
              label: t('Ne pas renvoyer'),
              onTap: () => s.resolveOutbox(mail, resend: false),
            ),
          ],
        ],
      ),
    );
  }
}

class _Tag extends StatelessWidget {
  final String label;
  final Color? dot;
  final bool accent;
  const _Tag({required this.label, this.dot, this.accent = false});
  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
      decoration: BoxDecoration(
        border: Border.all(color: accent ? p.accent : p.line),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          if (dot != null) ...[Square(7, dot!), const SizedBox(width: 6)],
          Mono(
            label,
            size: 10.5,
            ls: .12,
            color: accent ? p.accent : p.muted,
            height: null,
          ),
        ],
      ),
    );
  }
}

class _Thread extends StatelessWidget {
  final Mail mail;
  final Msg msg;
  final bool old, narrow, mobile;
  final VoidCallback toggle;
  const _Thread({
    required this.mail,
    required this.msg,
    required this.old,
    required this.toggle,
    required this.narrow,
    required this.mobile,
  });

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final s = context.watch<MailStore>();
    final to = isMine(msg.from.email)
        ? (!isMine(mail.from.email)
              ? s.shown(mail.from)
              : mail.to.map(s.shown).join(', '))
        : t('moi');
    final fromName = s.shown(msg.from);
    final bodyLeft = narrow ? 24.0 : 74.0;
    final info = Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          fromName,
          style: Fonts.sans(
            size: 15,
            w: FontWeight.w600,
            color: p.text,
            height: 1.3,
          ),
        ),
        Text(
          t('<{0}> · à {1}', [msg.from.email, to]),
          maxLines: 1,
          overflow: TextOverflow.ellipsis,
          style: Fonts.sans(size: 13.5, color: p.dim),
        ),
      ],
    );
    final time = Mono(fullDate(msg.date), ls: .08, height: null);
    final chev = AnimatedRotation(
      turns: old ? -.25 : 0,
      duration: const Duration(milliseconds: 150),
      child: Ico('chev', size: 14, color: p.dim),
    );

    return Container(
      margin: const EdgeInsets.only(top: 20),
      decoration: BoxDecoration(
        color: p.surface,
        border: Border.all(color: p.line),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Hov(
            onTap: toggle,
            builder: (_, _) => Opacity(
              opacity: old ? .75 : 1,
              child: Padding(
                padding: const EdgeInsets.all(16).copyWith(left: 20, right: 20),
                child: mobile
                    ? Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Row(
                            children: [
                              Avatar(fromName, size: 40, font: 11),
                              const SizedBox(width: 14),
                              Expanded(child: info),
                            ],
                          ),
                          const SizedBox(height: 2),
                          chev,
                          Padding(
                            padding: const EdgeInsets.only(left: 52, top: 2),
                            child: time,
                          ),
                        ],
                      )
                    : Row(
                        children: [
                          Avatar(fromName, size: 40, font: 11),
                          const SizedBox(width: 14),
                          Expanded(child: info),
                          const SizedBox(width: 14),
                          time,
                          const SizedBox(width: 14),
                          chev,
                        ],
                      ),
              ),
            ),
          ),
          AnimatedSize(
            duration: const Duration(milliseconds: 220),
            curve: Curves.easeOutCubic,
            alignment: Alignment.topCenter,
            child: ClipRect(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  if (!old) ...[
                    Padding(
                      padding: EdgeInsets.fromLTRB(bodyLeft, 2, 24, 24),
                      child: _BodySwitch(msg: msg, p: p),
                    ),
                    if (msg.att.isNotEmpty)
                      Padding(
                        padding: EdgeInsets.fromLTRB(bodyLeft, 0, 24, 22),
                        child: Wrap(
                          spacing: 12,
                          runSpacing: 12,
                          children: [
                            for (final (fi, f) in msg.att.indexed)
                              Hov(
                                tip: t('Lire le document'),
                                onTap: () =>
                                    showAttachmentViewer(context, msg.att, fi),
                                builder: (_, h) => DashedBox(
                                  color: h ? p.accent : p.line,
                                  fill: p.bg,
                                  padding: const EdgeInsets.symmetric(
                                    horizontal: 14,
                                    vertical: 10,
                                  ),
                                  child: Row(
                                    mainAxisSize: MainAxisSize.min,
                                    children: [
                                      Ico(
                                        f.t == 'PNG' ? 'img' : 'file',
                                        color: h ? p.text : p.muted,
                                      ),
                                      const SizedBox(width: 12),
                                      Column(
                                        crossAxisAlignment:
                                            CrossAxisAlignment.start,
                                        mainAxisSize: MainAxisSize.min,
                                        children: [
                                          Text(
                                            f.n,
                                            style: Fonts.sans(
                                              size: 15,
                                              color: h ? p.text : p.muted,
                                            ),
                                          ),
                                          Mono(
                                            '${f.t} · ${f.s}',
                                            size: 10,
                                            ls: .12,
                                            height: null,
                                          ),
                                        ],
                                      ),
                                    ],
                                  ),
                                ),
                              ),
                          ],
                        ),
                      ),
                  ],
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}

/// Demande confirmation avant de se désabonner, en disant comment cela se fera.
Future<void> _confirmUnsubscribe(BuildContext context, Mail mail) async {
  final s = context.read<MailStore>();
  final p = context.pal;
  final u = s.unsubFor(mail);
  if (u == null) return;
  final name = s.shown(mail.from);
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
                t('Se désabonner de {0} ?', [name]),
                style: Fonts.sans(size: 16, w: FontWeight.w600, color: p.text),
              ),
              const SizedBox(height: 12),
              Text(
                u.inApp
                    ? t(
                        'MIRA envoie la demande de désabonnement à {0} ; tu ne devrais plus recevoir ses envois à cette adresse.',
                        [mail.from.email],
                      )
                    : t(
                        '{0} demande de finir le désabonnement sur son site : la page va s’ouvrir dans ton navigateur.',
                        [name],
                      ),
                style: Fonts.sans(size: 14, color: p.dim, height: 1.5),
              ),
              const SizedBox(height: 20),
              Row(
                mainAxisAlignment: MainAxisAlignment.end,
                children: [
                  Btn(
                    label: t('Annuler'),
                    onTap: () => Navigator.pop(ctx, false),
                  ),
                  const SizedBox(width: 10),
                  Btn(
                    label: t('Se désabonner'),
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
  if (ok == true) await s.unsubscribe(mail);
}

/// Corps d'un message : squelette pendant le chargement, puis fondu vers le texte ou le HTML.
class _BodySwitch extends StatelessWidget {
  final Msg msg;
  final Pal p;
  const _BodySwitch({required this.msg, required this.p});

  @override
  Widget build(BuildContext context) {
    final real = context.read<MailStore>().source != null && msg.ref != null;
    final loading = real && !msg.loaded;
    final base = Fonts.sans(size: 16, color: p.body, height: 1.75);
    final child = loading
        ? const BodySkeleton(key: ValueKey('loading'))
        : msg.html != null
        ? MailHtml(key: ValueKey('html-${msg.ref}'), msg: msg, base: base)
        : KeyedSubtree(
            key: ValueKey('text-${msg.ref}'),
            child: blocksView(msg.body, base, p),
          );
    return AnimatedSwitcher(
      duration: const Duration(milliseconds: 160),
      switchInCurve: Curves.easeOut,
      child: child,
    );
  }
}

const _deckSize = 5;

/// Pile de conversations : la courante est visible, les autres restent montées (et mises en page) hors écran.
class _Deck extends StatelessWidget {
  final Mail current;
  final List<String> ids;
  const _Deck({required this.current, required this.ids});

  @override
  Widget build(BuildContext context) {
    final store = context.read<MailStore>();
    return Stack(
      fit: StackFit.expand,
      children: [
        for (final id in ids)
          if (id == current.id || store.byId(id) != null)
            Offstage(
              key: ValueKey('deck-$id'),
              offstage: id != current.id,
              child: TickerMode(
                enabled: id == current.id,
                child: _DeckFade(
                  active: id == current.id,
                  child: _MailBody(
                    key: ValueKey('body-$id'),
                    mail: id == current.id ? current : store.byId(id)!,
                  ),
                ),
              ),
            ),
      ],
    );
  }
}

/// Fondu court quand une conversation déjà prête repasse au premier plan.
class _DeckFade extends StatefulWidget {
  final bool active;
  final Widget child;
  const _DeckFade({required this.active, required this.child});

  @override
  State<_DeckFade> createState() => _DeckFadeState();
}

class _DeckFadeState extends State<_DeckFade>
    with SingleTickerProviderStateMixin {
  late final AnimationController _c = AnimationController(
    vsync: this,
    duration: const Duration(milliseconds: 150),
    value: 1,
  );

  @override
  void didUpdateWidget(_DeckFade old) {
    super.didUpdateWidget(old);
    if (widget.active && !old.active) _c.forward(from: .4);
  }

  @override
  void dispose() {
    _c.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) =>
      FadeTransition(opacity: _c, child: widget.child);
}

/// Contenu défilant d'une conversation (objet, messages repliables).
class _MailBody extends StatefulWidget {
  final Mail mail;
  const _MailBody({super.key, required this.mail});

  @override
  State<_MailBody> createState() => _MailBodyState();
}

class _MailBodyState extends State<_MailBody> {
  late Set<int> old;
  String sig = '';

  void _sync() {
    final k = '${widget.mail.id}:${widget.mail.msgs.length}';
    if (k != sig) {
      sig = k;
      old = {for (var i = 0; i < widget.mail.msgs.length - 1; i++) i};
    }
  }

  @override
  Widget build(BuildContext context) {
    _sync();
    final p = context.pal;
    final m = widget.mail;
    final w = MediaQuery.sizeOf(context).width;
    final narrow = w <= 1100, mobile = w <= 760;
    final hp = narrow ? 16.0 : 32.0;
    final f = folderById(m.folder);
    final chips = <Widget>[
      for (final l in m.labels)
        if (context.read<MailStore>().labelDef(l) case final d?)
          _Tag(label: context.read<MailStore>().labelTitle(l), dot: Color(d.c)),
      if (m.flags.contains('needs')) _Tag(label: t('À répondre'), accent: true),
      if (m.flags.contains('waiting')) _Tag(label: t('En attente')),
    ];
    return ScrollArea(
      padding: EdgeInsets.fromLTRB(hp, 28, hp, 40),
      child: SelectionArea(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Ico(f.icon, size: 14, color: p.dim),
                const SizedBox(width: 8),
                Flexible(
                  child: Mono(
                    '${t(f.n)}${m.msgs.length > 1 ? ' · ${tn(m.msgs.length, 'message', 'messages')}' : ''}',
                  ),
                ),
              ],
            ),
            Padding(
              padding: const EdgeInsets.only(top: 10),
              child: Text(
                m.subject.toUpperCase(),
                style: Fonts.display(
                  size: narrow && mobile ? 28 : (narrow ? 28 : 38),
                  height: 1.08,
                  color: p.text,
                ),
              ),
            ),
            Padding(
              padding: const EdgeInsets.only(top: 12),
              child: Wrap(spacing: 6, runSpacing: 6, children: chips),
            ),
            for (var i = 0; i < m.msgs.length; i++)
              FadeSlideIn(
                delay: Duration(milliseconds: 60 * (i > 4 ? 4 : i)),
                from: const Offset(0, 12),
                child: _Thread(
                  mail: m,
                  msg: m.msgs[i],
                  old: old.contains(i),
                  toggle: () => setState(
                    () => old.contains(i) ? old.remove(i) : old.add(i),
                  ),
                  narrow: narrow,
                  mobile: mobile,
                ),
              ),
          ],
        ),
      ),
    );
  }
}
