import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../data/format.dart';
import '../data/models.dart';
import '../state/store.dart';
import '../theme/app_theme.dart';
import '../theme/icons.dart';
import 'native_cover.dart';
import 'widgets.dart';
import '../i18n/i18n.dart';

// ============================ Popovers ============================

class _PopLayout extends SingleChildLayoutDelegate {
  final Rect anchor;
  final bool alignRight;
  final double maxWidth;
  _PopLayout(this.anchor, this.alignRight, this.maxWidth);

  @override
  BoxConstraints getConstraintsForChild(BoxConstraints c) => BoxConstraints(
    maxWidth: maxWidth.clamp(0, c.maxWidth - 16),
    maxHeight: c.maxHeight - 16,
  );

  @override
  Offset getPositionForChild(Size size, Size child) {
    var left = alignRight ? anchor.right - child.width : anchor.left;
    left = left.clamp(
      8.0,
      size.width - child.width - 8 < 8 ? 8.0 : size.width - child.width - 8,
    );
    var top = anchor.bottom + 6;
    if (top + child.height > size.height - 8) {
      top = (anchor.top - child.height - 6).clamp(8.0, double.infinity);
    }
    return Offset(left, top);
  }

  @override
  bool shouldRelayout(_PopLayout o) =>
      o.anchor != anchor || o.maxWidth != maxWidth;
}

class Pops {
  static OverlayEntry? _entry;
  static String? _key;

  static bool get isOpen => _entry != null;

  static void close() {
    _entry?.remove();
    _entry = null;
    _key = null;
  }

  /// Ouvre (ou ferme, si déjà ouvert) un popover ancré sur [anchor].
  static void show(
    BuildContext anchor,
    String key,
    Widget Function(BuildContext ctx, VoidCallback close) builder, {
    bool alignRight = false,
    double maxWidth = 320,
  }) {
    if (_key == key) return close();
    close();
    final box = anchor.findRenderObject() as RenderBox;
    final rect = box.localToGlobal(Offset.zero) & box.size;
    final overlay = Overlay.of(anchor, rootOverlay: true);
    _key = key;
    _entry = OverlayEntry(
      builder: (ctx) => Stack(
        children: [
          Positioned.fill(
            child: Listener(
              behavior: HitTestBehavior.translucent,
              onPointerDown: (e) {
                if (!rect.contains(e.position)) close();
              },
            ),
          ),
          Positioned.fill(
            child: CustomSingleChildLayout(
              delegate: _PopLayout(rect, alignRight, maxWidth),
              child: CoversNativeViews(
                child: _PopBox(child: builder(ctx, close)),
              ),
            ),
          ),
        ],
      ),
    );
    overlay.insert(_entry!);
  }
}

class _PopBox extends StatelessWidget {
  final Widget child;
  const _PopBox({required this.child});

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return TweenAnimationBuilder<double>(
      tween: Tween(begin: 0, end: 1),
      duration: const Duration(milliseconds: 120),
      curve: Curves.easeOut,
      builder: (_, t, c) => Opacity(
        opacity: t,
        child: Transform.translate(offset: Offset(0, -4 * (1 - t)), child: c),
      ),
      child: Material(
        type: MaterialType.transparency,
        child: Container(
          constraints: const BoxConstraints(minWidth: 220),
          padding: const EdgeInsets.all(6),
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
          child: IntrinsicWidth(child: SingleChildScrollView(child: child)),
        ),
      ),
    );
  }
}

class PopTitle extends StatelessWidget {
  final String text;
  const PopTitle(this.text, {super.key});
  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.fromLTRB(12, 8, 12, 6),
    child: Mono(text, size: 10.5, ls: .18, height: null),
  );
}

class PopItem extends StatelessWidget {
  final Widget leading;
  final String label;
  final bool? on; // null : pas de coche
  final VoidCallback onTap;
  const PopItem({
    super.key,
    required this.leading,
    required this.label,
    this.on,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Hov(
      onTap: onTap,
      builder: (_, h) {
        final active = on == true;
        final fg = active ? p.accent : (h ? p.text : p.muted);
        return Container(
          color: h ? p.surface : Colors.transparent,
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 9),
          child: Row(
            children: [
              leading,
              if (leading is! SizedBox) const SizedBox(width: 12),
              Expanded(
                child: Text(label, style: Fonts.sans(size: 14, color: fg)),
              ),
              if (on != null) ...[
                SizedBox(width: 12),
                Opacity(
                  opacity: active ? 1 : 0,
                  child: Ico('check', size: 14, color: p.accent),
                ),
              ],
            ],
          ),
        );
      },
    );
  }
}

class PopSep extends StatelessWidget {
  const PopSep({super.key});
  @override
  Widget build(BuildContext context) => Container(
    height: 1,
    margin: EdgeInsets.symmetric(vertical: 6),
    color: context.pal.line,
  );
}

Widget popIco(String name, BuildContext c) => Ico(name, color: c.pal.muted);

// ---- contenus ----

void showSortPop(BuildContext anchor) {
  Pops.show(anchor, 'sort', (ctx, close) {
    return ListenableBuilder(
      listenable: ctx.read<MailStore>(),
      builder: (ctx, _) {
        final s = ctx.read<MailStore>();
        final items = [
          ('new', t('Plus récents')),
          ('old', t('Plus anciens')),
          ('unread', t('Non lus d’abord')),
        ];
        return Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          mainAxisSize: MainAxisSize.min,
          children: [
            PopTitle(t('Trier par')),
            for (final (k, n) in items)
              PopItem(
                leading: SizedBox.shrink(),
                label: n,
                on: s.sort == k,
                onTap: () {
                  close();
                  s.setSort(k);
                },
              ),
            PopSep(),
            PopTitle(t('Afficher seulement')),
            for (final (k, n) in [
              ('unread', t('Non lus')),
              ('att', t('Avec pièces jointes')),
            ])
              PopItem(
                key: ValueKey('only-$k'),
                leading: SizedBox.shrink(),
                label: n,
                on: s.only == k,
                onTap: () {
                  close();
                  s.setOnly(k);
                },
              ),
          ],
        );
      },
    );
  }, alignRight: true);
}

void showSnoozePop(BuildContext anchor) {
  Pops.show(anchor, 'snooze', (ctx, close) {
    final s = ctx.read<MailStore>();
    final items = [
      ('1m', t('Dans 1 minute (démo)')),
      ('1h', t('Dans 1 heure')),
      ('tom', t('Demain 08:00')),
      ('mon', t('Lundi 08:00')),
    ];
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      mainAxisSize: MainAxisSize.min,
      children: [
        PopTitle(t('Reporter jusqu’à…')),
        for (final (k, n) in items)
          PopItem(
            leading: popIco('clock', ctx),
            label: n,
            onTap: () {
              close();
              s.snooze(k);
            },
          ),
      ],
    );
  }, alignRight: true);
}

void showSchedPop(BuildContext anchor) {
  Pops.show(anchor, 'sched', (ctx, close) {
    final s = ctx.read<MailStore>();
    final items = ['Dans 1 heure', 'Demain 08:00', 'Lundi 08:00']; // clés FR
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      mainAxisSize: MainAxisSize.min,
      children: [
        PopTitle(t('Programmer l’envoi')),
        for (final n in items)
          PopItem(
            leading: popIco('clock', ctx),
            label: t(n),
            onTap: () {
              close();
              s.send(n.toLowerCase());
            },
          ),
      ],
    );
  });
}

void showLabelPop(BuildContext anchor) {
  Pops.show(anchor, 'label', (ctx, close) {
    return ListenableBuilder(
      listenable: ctx.read<MailStore>(),
      builder: (ctx, _) {
        final s = ctx.read<MailStore>();
        final ms = s.targets().map((i) => s.byId(i)!).toList();
        return Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          mainAxisSize: MainAxisSize.min,
          children: [
            PopTitle(t('Libellés')),
            for (final e in s.labelsForTargets().entries)
              PopItem(
                leading: Square(8, Color(e.value.c)),
                label: s.labelTitle(e.key),
                on: ms.isNotEmpty && ms.every((m) => m.labels.contains(e.key)),
                onTap: () => s.toggleLabel(e.key),
              ),
          ],
        );
      },
    );
  }, alignRight: true);
}

void showMorePop(BuildContext anchor) {
  Pops.show(anchor, 'more', (ctx, close) {
    final s = ctx.read<MailStore>();
    void run(VoidCallback f) {
      close();
      f();
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      mainAxisSize: MainAxisSize.min,
      children: [
        PopTitle(t('Actions')),
        PopItem(
          leading: popIco('check', ctx),
          label: t('Tout sélectionner'),
          onTap: () => run(s.selAll),
        ),
        PopItem(
          leading: popIco('x', ctx),
          label: t('Tout désélectionner'),
          onTap: () => run(s.selNone),
        ),
        PopItem(
          leading: popIco('inbox', ctx),
          label: t('Tout marquer comme lu'),
          onTap: () => run(s.readAllVisible),
        ),
        if (s.source != null && s.label == null)
          PopItem(
            leading: popIco('refresh', ctx),
            label: t('Resynchroniser ce dossier'),
            onTap: () => run(s.resyncFolder),
          ),
        if (s.folder == 'trash' || s.folder == 'spam')
          PopItem(
            leading: popIco('trash', ctx),
            label: t('Vider ce dossier'),
            onTap: () => run(s.emptyFolder),
          ),
        PopSep(),
        PopItem(
          leading: popIco('help', ctx),
          label: t('Raccourcis clavier'),
          onTap: () => run(s.openHelp),
        ),
      ],
    );
  }, alignRight: true);
}

// ============================ Toasts ============================

class ToastHost extends StatelessWidget {
  const ToastHost({super.key});

  @override
  Widget build(BuildContext context) {
    final s = context.watch<MailStore>();
    final w = MediaQuery.sizeOf(context).width;
    final mobile = w <= 760;
    return Positioned(
      left: mobile ? 12 : 272,
      right: mobile ? 12 : null,
      bottom: mobile ? 12 : 24,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          for (final t in s.toasts)
            Padding(
              key: ValueKey(t.id),
              padding: const EdgeInsets.only(top: 8),
              child: _ToastView(t),
            ),
        ],
      ),
    );
  }
}

class _ToastView extends StatelessWidget {
  final ToastItem t;
  const _ToastView(this.t);

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final s = context.read<MailStore>();
    final bg = t.err ? p.accent : p.toastBg;
    final fg = t.err ? p.onAccent : p.toastFg;
    return TweenAnimationBuilder<double>(
      tween: Tween(begin: 0, end: t.out ? 0 : 1),
      duration: Duration(milliseconds: t.out ? 250 : 200),
      builder: (_, v, child) => Opacity(
        opacity: v,
        child: Transform.translate(
          offset: Offset(0, (1 - v) * (t.out ? 8 : 10)),
          child: child,
        ),
      ),
      child: Material(
        type: MaterialType.transparency,
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
          decoration: BoxDecoration(
            color: bg,
            boxShadow: const [
              BoxShadow(
                color: Color(0x66000000),
                blurRadius: 30,
                offset: Offset(0, 10),
              ),
            ],
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Flexible(
                child: Text(t.msg, style: Fonts.sans(size: 14, color: fg)),
              ),
              if (t.label != null) ...[
                SizedBox(width: 18),
                Hov(
                  onTap: () {
                    s.killToast(t);
                    t.cb?.call();
                  },
                  builder: (_, _) => Mono(
                    t.label!,
                    size: 11,
                    w: FontWeight.w700,
                    ls: .14,
                    color: t.err ? fg : p.toastAccent,
                    height: null,
                  ),
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }
}

// ============================ Modale des raccourcis ============================

class HelpModal extends StatelessWidget {
  const HelpModal({super.key});

  @override
  Widget build(BuildContext context) {
    final s = context.watch<MailStore>();
    if (!s.showHelp) return SizedBox.shrink();
    final p = context.pal;
    final rows = [
      ('j / k', t('Message suivant / précédent')),
      (t('Entrée'), t('Ouvrir le message')),
      ('c', t('Nouveau message')),
      ('r', t('Répondre')),
      ('a', t('Répondre à tous')),
      ('f', t('Transférer')),
      ('e', t('Archiver')),
      ('#', t('Supprimer')),
      ('s', t('Favori')),
      ('u', t('Lu / non lu')),
      ('x', t('Sélectionner')),
      ('/', t('Rechercher')),
      (t('Ctrl + Entrée'), t('Envoyer le message')),
      if (s.source != null) ('F5 / Ctrl + R', t('Actualiser maintenant')),
      (t('Échap'), t('Fermer')),
    ];
    final w = MediaQuery.sizeOf(context).width;
    return Positioned.fill(
      child: CoversNativeViews(
        modal: true,
        child: GestureDetector(
          onTap: s.closeHelp,
          child: Container(
            color: const Color(0x8c000000),
            alignment: Alignment.center,
            child: GestureDetector(
              onTap: () {},
              child: FadeSlideIn(
                from: const Offset(0, 14),
                child: Material(
                  type: MaterialType.transparency,
                  child: Container(
                    width: (w * .92).clamp(0, 560).toDouble(),
                    padding: const EdgeInsets.all(28),
                    decoration: BoxDecoration(
                      color: p.bgSide,
                      border: Border.all(color: p.line),
                    ),
                    child: SingleChildScrollView(
                      child: Column(
                        mainAxisSize: MainAxisSize.min,
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Padding(
                            padding: const EdgeInsets.only(bottom: 18),
                            child: Text(
                              t('RACCOURCIS CLAVIER'),
                              style: Fonts.display(size: 28, color: p.text),
                            ),
                          ),
                          Table(
                            defaultColumnWidth: const IntrinsicColumnWidth(),
                            columnWidths: const {1: FlexColumnWidth()},
                            defaultVerticalAlignment:
                                TableCellVerticalAlignment.middle,
                            children: [
                              for (final (k, d) in rows)
                                TableRow(
                                  children: [
                                    Padding(
                                      padding: const EdgeInsets.only(
                                        right: 22,
                                        bottom: 10,
                                      ),
                                      child: Align(
                                        alignment: Alignment.centerLeft,
                                        child: Kbd(k),
                                      ),
                                    ),
                                    Padding(
                                      padding: const EdgeInsets.only(
                                        bottom: 10,
                                      ),
                                      child: Text(
                                        d,
                                        style: Fonts.sans(
                                          size: 14.5,
                                          color: p.muted,
                                        ),
                                      ),
                                    ),
                                  ],
                                ),
                            ],
                          ),
                          const SizedBox(height: 12),
                          Btn(label: t('Fermer'), onTap: s.closeHelp),
                        ],
                      ),
                    ),
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}

// ============================ Comptes ============================

class AccountAvatar extends StatelessWidget {
  final Account a;
  final double size;
  const AccountAvatar(this.a, {super.key, this.size = 22});

  @override
  Widget build(BuildContext context) => Container(
    width: size,
    height: size,
    alignment: Alignment.center,
    color: Color(colorOf(a.email)),
    child: Text(
      initials(a.name),
      style: Fonts.mono(
        size: size * .42,
        w: FontWeight.w700,
        color: const Color(0xFF1b1b1b),
        ls: 0,
        height: 1,
      ),
    ),
  );
}

/// Entrées du popover des comptes : hauteur, nombre au plus par colonne, largeur d'une colonne.
const _accountRowHeight = 52.0;
const _accountRowsPerColumn = 3;
const _accountColumnWidth = 260.0;

/// Entrées rangées en colonnes de trois au plus (autant de colonnes que l'écran en permet) : le popover s'élargit au lieu de s'allonger.
class _AccountColumns extends StatelessWidget {
  final List<Widget> children;
  const _AccountColumns({required this.children});

  @override
  Widget build(BuildContext context) {
    final room = ((MediaQuery.sizeOf(context).width - 40) / _accountColumnWidth)
        .floor()
        .clamp(1, 1 << 10);
    final wanted =
        (children.length + _accountRowsPerColumn - 1) ~/ _accountRowsPerColumn;
    final columns = wanted.clamp(1, room);
    final perColumn = (children.length + columns - 1) ~/ columns;
    return Row(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        for (var c = 0; c < columns; c++)
          SizedBox(
            width: _accountColumnWidth,
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: children.skip(c * perColumn).take(perColumn).toList(),
            ),
          ),
      ],
    );
  }
}

/// Liste déroulante des comptes (le compte actif est coché) + « Ajouter un compte ».
void showAccountPop(BuildContext anchor) {
  Pops.show(
    anchor,
    'account',
    (ctx, close) {
      return ListenableBuilder(
        listenable: ctx.read<MailStore>(),
        builder: (ctx, _) {
          final s = ctx.read<MailStore>();
          final p = ctx.pal;
          return Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            mainAxisSize: MainAxisSize.min,
            children: [
              PopTitle(t('Comptes')),
              _AccountColumns(
                children: [
                  Hov(
                    onTap: () {
                      close();
                      s.showAllAccounts();
                    },
                    builder: (_, h) => Container(
                      color: h ? p.surface : Colors.transparent,
                      height: _accountRowHeight,
                      padding: const EdgeInsets.symmetric(horizontal: 12),
                      child: Row(
                        children: [
                          Container(
                            width: 28,
                            height: 28,
                            alignment: Alignment.center,
                            color: p.accent,
                            child: Ico('inbox', size: 16, color: p.onAccent),
                          ),
                          const SizedBox(width: 12),
                          Expanded(
                            child: Column(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                Text(
                                  t('Toutes les boîtes'),
                                  style: Fonts.sans(
                                    size: 14,
                                    w: FontWeight.w500,
                                    color: s.unified ? p.accent : p.text,
                                    height: 1.3,
                                  ),
                                ),
                                Text(
                                  t('Boîte unifiée · {0} comptes', [
                                    s.accounts.length,
                                  ]),
                                  style: Fonts.sans(
                                    size: 12.5,
                                    color: p.dim,
                                    height: 1.3,
                                  ),
                                ),
                              ],
                            ),
                          ),
                          const SizedBox(width: 12),
                          Opacity(
                            opacity: s.unified ? 1 : 0,
                            child: Ico('check', size: 14, color: p.accent),
                          ),
                        ],
                      ),
                    ),
                  ),
                  for (final a in s.accounts)
                    Hov(
                      onTap: () {
                        close();
                        s.switchAccount(a.id);
                      },
                      builder: (_, h) {
                        final on = !s.unified && a.id == s.activeId;
                        return Container(
                          color: h ? p.surface : Colors.transparent,
                          height: _accountRowHeight,
                          padding: const EdgeInsets.symmetric(horizontal: 12),
                          child: Row(
                            children: [
                              AccountAvatar(a, size: 28),
                              const SizedBox(width: 12),
                              Expanded(
                                child: Column(
                                  crossAxisAlignment: CrossAxisAlignment.start,
                                  children: [
                                    Text(
                                      a.name,
                                      maxLines: 1,
                                      overflow: TextOverflow.ellipsis,
                                      style: Fonts.sans(
                                        size: 14,
                                        w: FontWeight.w500,
                                        color: on ? p.accent : p.text,
                                        height: 1.3,
                                      ),
                                    ),
                                    Text(
                                      a.email,
                                      maxLines: 1,
                                      overflow: TextOverflow.ellipsis,
                                      style: Fonts.sans(
                                        size: 12.5,
                                        color: p.dim,
                                        height: 1.3,
                                      ),
                                    ),
                                  ],
                                ),
                              ),
                              const SizedBox(width: 12),
                              Opacity(
                                opacity: on ? 1 : 0,
                                child: Ico('check', size: 14, color: p.accent),
                              ),
                            ],
                          ),
                        );
                      },
                    ),
                ],
              ),
              PopSep(),
              PopItem(
                leading: popIco('plus', ctx),
                label: t('Ajouter un compte'),
                onTap: () {
                  close();
                  s.source != null
                      ? s.openAddAccount()
                      : showAddAccountDialog(anchor);
                },
              ),
            ],
          );
        },
      );
    },
    alignRight: true,
    maxWidth: double.infinity,
  );
}

final _mailRe = RegExp(r'^[^\s@<>,;]+@[^\s@<>,;]+\.[^\s@<>,;]+$');

/// Boîte de dialogue « Ajouter un compte » (fictive : aucune connexion réelle).
Future<void> showAddAccountDialog(BuildContext context) async {
  final s = context.read<MailStore>();
  final name = TextEditingController();
  final email = TextEditingController();
  await showDialog<void>(
    context: context,
    builder: (ctx) {
      final p = ctx.pal;
      String? err;
      return StatefulBuilder(
        builder: (ctx, setState) {
          void submit() {
            final em = email.text.trim();
            if (!_mailRe.hasMatch(em)) {
              setState(() => err = t('Entre une adresse e-mail valide.'));
              return;
            }
            if (!s.addAccount(name.text, em)) {
              setState(() => err = t('Ce compte est déjà connecté.'));
              return;
            }
            Navigator.pop(ctx);
          }

          Widget field(
            String label,
            TextEditingController c, {
            String? hint,
            bool autofocus = false,
            bool bad = false,
          }) => Padding(
            padding: const EdgeInsets.only(bottom: 14),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Mono(label, height: null),
                const SizedBox(height: 6),
                Container(
                  padding: const EdgeInsets.symmetric(
                    horizontal: 12,
                    vertical: 10,
                  ),
                  decoration: BoxDecoration(
                    color: p.surface,
                    border: Border.all(color: bad ? p.accent : p.line),
                  ),
                  child: TextField(
                    controller: c,
                    autofocus: autofocus,
                    onSubmitted: (_) => submit(),
                    onChanged: (_) {
                      if (err != null) setState(() => err = null);
                    },
                    cursorColor: p.accent,
                    cursorWidth: 1,
                    style: Fonts.sans(size: 14.5, color: p.text),
                    decoration: InputDecoration(
                      isDense: true,
                      border: InputBorder.none,
                      contentPadding: EdgeInsets.zero,
                      hintText: hint,
                      hintStyle: Fonts.sans(size: 14.5, color: p.dim),
                    ),
                  ),
                ),
              ],
            ),
          );

          return Dialog(
            backgroundColor: p.bgSide,
            shape: RoundedRectangleBorder(side: BorderSide(color: p.line)),
            child: Padding(
              padding: const EdgeInsets.all(28),
              child: ConstrainedBox(
                constraints: const BoxConstraints(maxWidth: 460),
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      t('AJOUTER UN COMPTE'),
                      style: Fonts.display(size: 28, color: p.text),
                    ),
                    const SizedBox(height: 8),
                    Text(
                      t(
                        'Compte fictif : aucune connexion réelle n’est effectuée.',
                      ),
                      style: Fonts.sans(size: 13.5, color: p.dim),
                    ),
                    const SizedBox(height: 20),
                    field(t('Nom'), name, hint: 'Noah Toffa', autofocus: true),
                    field(
                      t('Adresse e-mail'),
                      email,
                      hint: 'nom@exemple.com',
                      bad: err != null,
                    ),
                    if (err != null)
                      Padding(
                        padding: const EdgeInsets.only(bottom: 14),
                        child: Text(
                          err!,
                          style: Fonts.sans(size: 13.5, color: p.accent),
                        ),
                      ),
                    Row(
                      mainAxisAlignment: MainAxisAlignment.end,
                      children: [
                        Btn(
                          label: t('Annuler'),
                          onTap: () => Navigator.pop(ctx),
                        ),
                        const SizedBox(width: 10),
                        Btn(
                          icon: 'plus',
                          label: t('Ajouter'),
                          go: true,
                          onTap: submit,
                        ),
                      ],
                    ),
                  ],
                ),
              ),
            ),
          );
        },
      );
    },
  );
}
