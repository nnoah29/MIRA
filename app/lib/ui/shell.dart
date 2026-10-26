import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:provider/provider.dart';

import '../state/store.dart';
import '../theme/app_theme.dart';
import '../theme/icons.dart';
import 'assistant_panel.dart';
import 'composer.dart';
import 'mail_list.dart';
import 'overlays.dart';
import 'push_stack.dart';
import 'side_drawer.dart';
import 'reader.dart';
import 'add_account_panel.dart';
import 'settings_panel.dart';
import 'sidebar.dart';
import 'sync_banner.dart';
import 'widgets.dart';
import '../i18n/i18n.dart';

class Shell extends StatefulWidget {
  const Shell({super.key});
  @override
  State<Shell> createState() => _ShellState();
}

class _ShellState extends State<Shell> with WidgetsBindingObserver {
  late final MailStore store = context.read<MailStore>();

  @override
  void initState() {
    super.initState();
    HardwareKeyboard.instance.addHandler(_onKey);
    WidgetsBinding.instance.addObserver(this);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      store.narrow = MediaQuery.sizeOf(context).width <= 1100;
      store.start();
    });
  }

  @override
  void dispose() {
    HardwareKeyboard.instance.removeHandler(_onKey);
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) store.refreshNow(quiet: true);
  }

  bool _editing() {
    final f = FocusManager.instance.primaryFocus;
    if (f == null || f.context == null) return false;
    return f.context!.findAncestorWidgetOfExactType<EditableText>() != null ||
        f.context!.widget is EditableText;
  }

  bool _onKey(KeyEvent e) {
    if (e is KeyUpEvent) return false;
    final s = store;
    final k = e.logicalKey;
    final kb = HardwareKeyboard.instance;
    final ctrl = kb.isControlPressed || kb.isMetaPressed;

    if (k == LogicalKeyboardKey.f5 ||
        (ctrl && k == LogicalKeyboardKey.keyR && !_editing())) {
      s.refreshNow();
      return true;
    }

    if (k == LogicalKeyboardKey.escape) {
      if (s.showHelp) {
        s.closeHelp();
        return true;
      }
      if (Pops.isOpen) {
        Pops.close();
        return true;
      }
      if (ModalRoute.of(context)?.isCurrent == false) {
        return false; // boîte de dialogue ouverte
      }
      if (s.c != null) {
        s.closeCompose(true);
        return true;
      }
      if (s.searchFocus.hasFocus) {
        if (s.hasQueryText) {
          s.clearQuery();
        } else {
          s.searchFocus.unfocus();
        }
        return true;
      }
      if (s.sel.isNotEmpty) {
        s.selNone();
        return true;
      }
      if (s.open != null ||
          s.view == PaneView.assistant ||
          s.view == PaneView.settings ||
          s.view == PaneView.addAccount) {
        s.closePane();
        return true;
      }
      return false;
    }
    if (s.c != null &&
        ctrl &&
        (k == LogicalKeyboardKey.enter ||
            k == LogicalKeyboardKey.numpadEnter)) {
      s.send();
      return true;
    }
    if (_editing() || ctrl || kb.isAltPressed) return false;
    if (ModalRoute.of(context)?.isCurrent == false) return false;
    final ch = e.character;
    if (k == LogicalKeyboardKey.enter) {
      if (s.open == null) {
        s.step(1);
        return true;
      }
      return false;
    }
    if (k == LogicalKeyboardKey.delete) {
      s.trash();
      return true;
    }
    switch (ch) {
      case 'j':
        s.step(1);
      case 'k':
        s.step(-1);
      case 'c':
        s.openCompose(const ComposeInit());
      case 'r':
        s.replyTo('reply');
      case 'a':
        s.replyTo('all');
      case 'f':
        s.replyTo('fwd');
      case 'e':
        s.archive();
      case '#':
        s.trash();
      case 's':
        s.toggleStar();
      case 'u':
        s.toggleRead();
      case 'x':
        if (s.open != null) s.toggleSel(s.open!);
      case '/':
        s.searchFocus.requestFocus();
      case '?':
        s.openHelp();
      default:
        return false;
    }
    return true;
  }

  @override
  Widget build(BuildContext context) {
    final s = context.watch<MailStore>();
    final p = context.pal;
    final w = MediaQuery.sizeOf(context).width;
    final narrow = w <= 1100;
    final mobile = w <= 760;
    s.narrow = narrow;
    final paneOpen = s.view != PaneView.empty;

    Widget split;
    if (!narrow) {
      split = Row(
        children: [
          SizedBox(width: 430, child: MailsColumn()),
          const Expanded(child: PaneView_()),
        ],
      );
    } else {
      split = PushStack(
        base: MailsColumn(),
        detail: paneOpen ? PaneSwitcher(child: paneChild(s)) : null,
        onBack: s.closePane,
      );
    }

    final main = Column(
      children: [
        const Topbar(),
        const SyncBanner(),
        Expanded(child: split),
      ],
    );

    final unread = s.navCount('inbox');
    return Title(
      title: '${unread > 0 ? '($unread) ' : ''}${t('Ma boîte')} — MIRA', // i18n:ignore
      color: p.bg,
      child: PopScope(
        canPop: !(s.navOpen || (narrow && paneOpen)),
        onPopInvokedWithResult: (didPop, _) {
          if (didPop) return;
          if (s.navOpen) {
            s.closeNav();
          } else if (!(s.paneBack?.call() ?? false)) {
            s.closePane();
          }
        },
        child: Scaffold(
          backgroundColor: p.bg,
          // Mobile : rien sous la barre d'état ni sous la barre de gestes (sans effet sur le bureau).
          body: SafeArea(
            child: DefaultTextStyle(
              style: Fonts.sans(size: 15, color: p.text),
              child: Stack(
                children: [
                  Row(
                    children: [
                      if (!mobile) const Sidebar(),
                      Expanded(child: main),
                    ],
                  ),
                  if (mobile)
                    Positioned.fill(
                      child: SideDrawer(
                        open: s.navOpen,
                        edgeEnabled: !paneOpen,
                        onOpen: s.openNav,
                        onClose: s.closeNav,
                        width: (w * .86).clamp(0, 300).toDouble(),
                        child: Sidebar(
                          width: (w * .86).clamp(0, 300).toDouble(),
                        ),
                      ),
                    ),
                  const ToastHost(),
                  HelpModal(),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}

/// Contenu du panneau droit selon la vue active : lecteur, rédaction, assistant, paramètres ou état vide.
Widget paneChild(MailStore s) {
  switch (s.view) {
    case PaneView.compose:
      return s.c == null
          ? const SizedBox.shrink()
          : Composer(s.c!, key: ObjectKey(s.c));
    case PaneView.assistant:
      return const AssistantPanel(key: ValueKey('assistant'));
    case PaneView.settings:
      return const SettingsPanel(key: ValueKey('settings'));
    case PaneView.addAccount:
      return const AddAccountPanel(key: ValueKey('add-account'));
    case PaneView.read:
      final m = s.byId(s.open);
      return m != null
          ? Reader(m, key: const ValueKey('reader'))
          : const _Empty(key: ValueKey('empty'));
    case PaneView.empty:
      return const _Empty(key: ValueKey('empty'));
  }
}

/// Fondu + léger glissement à chaque changement de contenu du panneau.
class PaneSwitcher extends StatelessWidget {
  final Widget child;
  const PaneSwitcher({super.key, required this.child});

  @override
  Widget build(BuildContext context) => AnimatedSwitcher(
    duration: const Duration(milliseconds: 200),
    switchInCurve: Curves.easeOutCubic,
    transitionBuilder: (c, a) => FadeTransition(
      opacity: a,
      child: SlideTransition(
        position: Tween(
          begin: const Offset(.012, 0),
          end: Offset.zero,
        ).animate(a),
        child: c,
      ),
    ),
    layoutBuilder: (current, previous) =>
        Stack(fit: StackFit.expand, children: [...previous, ?current]),
    child: child,
  );
}

/// Panneau droit des écrans larges.
// ignore: camel_case_types
class PaneView_ extends StatelessWidget {
  const PaneView_({super.key});

  @override
  Widget build(BuildContext context) =>
      PaneSwitcher(child: paneChild(context.watch<MailStore>()));
}

class _Empty extends StatelessWidget {
  const _Empty({super.key});
  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final s = context.read<MailStore>();
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Ico('inbox', size: 44, stroke: 1.1, color: p.dim),
          const SizedBox(height: 12),
          Mono(t('Sélectionne un message')),
          const SizedBox(height: 12),
          Text.rich(
            TextSpan(
              style: Fonts.sans(size: 15, color: p.dim),
              children: [
                TextSpan(text: t('ou écris-en un avec la touche ')),
                WidgetSpan(
                  alignment: PlaceholderAlignment.middle,
                  child: const Kbd('c'),
                ),
              ],
            ),
          ),
          const SizedBox(height: 22),
          Btn(
            icon: 'pen',
            label: t('Nouveau message'),
            go: true,
            onTap: () => s.openCompose(const ComposeInit()),
          ),
        ],
      ),
    );
  }
}
