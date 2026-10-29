import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:provider/provider.dart';

import '../data/format.dart';
import '../state/assistant.dart';
import '../state/store.dart';
import '../theme/app_theme.dart';
import '../theme/icons.dart';
import 'ai_consent_dialog.dart';
import 'chat_markdown.dart';
import 'demo_banner.dart';
import 'widgets.dart';
import '../i18n/i18n.dart';

class AssistantPanel extends StatefulWidget {
  const AssistantPanel({super.key});
  @override
  State<AssistantPanel> createState() => _AssistantPanelState();
}

class _AssistantPanelState extends State<AssistantPanel> {
  final input = TextEditingController();
  final focus = FocusNode();
  final thread = ScrollController();
  int lastLen = -1;
  String lastShown = '';

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted) return;
      focus.requestFocus();
      context.read<MailStore>().chatCtl.refresh();
    });
  }

  @override
  void dispose() {
    input.dispose();
    focus.dispose();
    thread.dispose();
    super.dispose();
  }

  Future<void> _send(MailStore s, [String? q]) async {
    final t = q ?? input.text;
    if (t.trim().isEmpty || s.busy) return;
    input.clear();
    await s.sendChat(t);
    if (mounted) await _askConsent(s);
  }

  Future<void> _askConsent(MailStore s) async {
    final chat = s.chatCtl;
    final name = chat.consentProviderName;
    if (chat.consentConnectionId == null || name == null) return;
    final agreed = await showAiConsentDialog(
      context,
      name,
      host: chat.consentHost,
    );
    if (agreed) {
      await chat.grantConsent();
    } else {
      chat.declineConsent();
    }
  }

  /// Texte du bandeau de contexte : sujet du mail ouvert, mail retiré ou aucun message.
  String _ctxLabel(MailStore s, String? subject) {
    if (subject == null) {
      return t(
        ' · aucun message sélectionné. Clique sur un mail dans la liste.',
      );
    }
    return s.ctxDropped
        ? t(' · retiré : l’assistant ne voit pas ce mail.')
        : ' · $subject';
  }

  /// Boutons du bandeau : ouvrir ou retirer le contexte, ou le rétablir une fois retiré.
  List<Widget> _ctxActions(MailStore s, bool hasMail) {
    if (!hasMail) return const [];
    const pad = EdgeInsets.symmetric(horizontal: 10, vertical: 6);
    if (s.ctxDropped) {
      return [
        Btn(
          key: const ValueKey('ctx-restore'),
          label: t('Rétablir'),
          padding: pad,
          fontSize: 9.5,
          onTap: s.restoreCtx,
        ),
      ];
    }
    return [
      Btn(
        key: const ValueKey('ctx-open'),
        label: t('Ouvrir'),
        padding: pad,
        fontSize: 9.5,
        onTap: s.openCtx,
      ),
      const SizedBox(width: 6),
      IBtn(
        'x',
        key: const ValueKey('ctx-drop'),
        tip: t('Retirer le contexte'),
        onTap: s.dropCtx,
      ),
    ];
  }

  @override
  Widget build(BuildContext context) {
    final s = context.watch<MailStore>();
    final p = context.pal;
    final w = MediaQuery.sizeOf(context).width;
    final narrow = w <= 1100;
    final hp = narrow ? 16.0 : 32.0;
    final m = s.byId(s.open);
    final demo = s.chatCtl.demo;

    // Défilement automatique en bas à chaque mise à jour du fil.
    final sig =
        s.chat.length * 100000 +
        (s.chat.isEmpty ? 0 : s.chat.last.shown.length);
    if (sig != lastLen) {
      lastLen = sig;
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (thread.hasClients) thread.jumpTo(thread.position.maxScrollExtent);
      });
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Padding(
          padding: EdgeInsets.fromLTRB(hp, 24, hp, 12),
          child: Row(
            children: [
              Container(
                width: 30,
                height: 30,
                color: p.accent,
                alignment: Alignment.center,
                child: Ico('bot', color: p.onAccent),
              ),
              const SizedBox(width: 12),
              Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    t('Assistant IA'),
                    style: Fonts.sans(
                      size: 15,
                      w: FontWeight.w500,
                      color: p.text,
                      height: 1.2,
                    ),
                  ),
                  Row(
                    children: [
                      Container(
                        width: 6,
                        height: 6,
                        decoration: const BoxDecoration(
                          color: Color(0xFF8bc48a),
                          shape: BoxShape.circle,
                        ),
                      ),
                      const SizedBox(width: 6),
                      Mono(
                        demo
                            ? t('Démonstration')
                            : (s.chatCtl.connectionLabel ?? t('En ligne')),
                        height: null,
                      ),
                    ],
                  ),
                ],
              ),
              const Spacer(),
              Btn(
                icon: 'plus',
                label: t('Nouvelle conversation'),
                onTap: s.clearChat,
              ),
              const SizedBox(width: 12),
              IBtn('x', tip: t('Fermer'), onTap: s.closePane),
            ],
          ),
        ),
        if (demo)
          Padding(
            padding: EdgeInsets.fromLTRB(hp, 0, hp, 8),
            child: DemoBanner(
              onSettings: s.source == null
                  ? null
                  : () => s.showView(PaneView.settings),
            ),
          ),
        Container(
          margin: EdgeInsets.fromLTRB(hp, 4, hp, 0),
          child: DashedBox(
            color: p.line,
            padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
            child: Row(
              children: [
                KeyedSubtree(
                  key: const ValueKey('ctx-icon'),
                  child: Ico('inbox', color: p.muted),
                ),
                const SizedBox(width: 12),
                Mono(t('Contexte'), color: p.accent, height: null),
                Expanded(
                  child: Text(
                    _ctxLabel(s, m?.subject),
                    key: const ValueKey('ctx-text'),
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: Fonts.sans(size: 14, color: p.muted),
                  ),
                ),
                ..._ctxActions(s, m != null),
              ],
            ),
          ),
        ),
        Expanded(
          child: ListView(
            controller: thread,
            padding: EdgeInsets.fromLTRB(
              hp,
              narrow ? 16 : 24,
              hp,
              narrow ? 16 : 24,
            ),
            children: [
              for (var i = 0; i < s.chat.length; i++)
                FadeSlideIn(
                  from: Offset(s.chat[i].role == 'me' ? 14 : -14, 6),
                  child: Padding(
                    padding: EdgeInsets.only(top: i == 0 ? 0 : 16),
                    child: Align(
                      alignment: s.chat[i].role == 'me'
                          ? Alignment.centerRight
                          : Alignment.centerLeft,
                      child: LayoutBuilder(
                        builder: (_, box) => _Bubble(
                          s.chat[i],
                          i,
                          maxW: (box.maxWidth * .88).clamp(0, 720).toDouble(),
                        ),
                      ),
                    ),
                  ),
                ),
            ],
          ),
        ),
        Padding(
          padding: EdgeInsets.fromLTRB(hp, 0, hp, 14),
          child: Wrap(
            spacing: 8,
            runSpacing: 8,
            children: [
              for (final sg in suggestions)
                Hov(
                  onTap: () => _send(s, sg[1]),
                  builder: (_, h) => Container(
                    padding: const EdgeInsets.symmetric(
                      horizontal: 11,
                      vertical: 7,
                    ),
                    decoration: BoxDecoration(
                      border: Border.all(color: h ? p.accent : p.line),
                    ),
                    child: Mono(
                      sg[0],
                      ls: .12,
                      color: h ? p.accent : p.muted,
                      height: null,
                    ),
                  ),
                ),
            ],
          ),
        ),
        Container(
          margin: EdgeInsets.fromLTRB(hp, 0, hp, 24),
          decoration: BoxDecoration(
            color: p.surface,
            border: Border.all(color: p.line),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Padding(
                padding: const EdgeInsets.symmetric(
                  horizontal: 16,
                  vertical: 14,
                ),
                child: CallbackShortcuts(
                  bindings: {
                    const SingleActivator(LogicalKeyboardKey.enter): () =>
                        _send(s),
                  },
                  child: TextField(
                    controller: input,
                    focusNode: focus,
                    minLines: 2,
                    maxLines: 2,
                    cursorColor: p.accent,
                    cursorWidth: 1,
                    style: Fonts.sans(size: 15, color: p.text),
                    decoration: InputDecoration(
                      isDense: true,
                      border: InputBorder.none,
                      contentPadding: EdgeInsets.zero,
                      hintText: t('Demande quelque chose à l’assistant…'),
                      hintStyle: Fonts.sans(size: 15, color: p.dim),
                    ),
                  ),
                ),
              ),
              Container(
                padding: const EdgeInsets.fromLTRB(16, 8, 10, 8),
                decoration: BoxDecoration(
                  border: Border(top: BorderSide(color: p.lineSoft)),
                ),
                child: Row(
                  children: [
                    Ico('spark', size: 13, color: p.dim),
                    const SizedBox(width: 8),
                    Mono(
                      demo ? t('Réponses simulées') : t('Réponses de l’IA'),
                      height: null,
                    ),
                    const Spacer(),
                    if (s.busy)
                      Btn(
                        key: const ValueKey('chat-stop'),
                        icon: 'x',
                        label: t('Arrêter'),
                        onTap: s.stopChat,
                      )
                    else
                      Hov(
                        onTap: () => _send(s),
                        builder: (_, _) => Container(
                          key: const ValueKey('chat-send'),
                          width: 34,
                          height: 34,
                          color: p.accent,
                          alignment: Alignment.center,
                          child: Ico('send', color: p.onAccent),
                        ),
                      ),
                  ],
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }
}

class _Bubble extends StatelessWidget {
  final ChatMsg msg;
  final int idx;
  final double maxW;
  const _Bubble(this.msg, this.idx, {required this.maxW});

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final s = context.read<MailStore>();
    final me = msg.role == 'me';
    final base = Fonts.sans(
      size: 15,
      color: me ? p.text : p.muted,
      height: 1.65,
    );
    final shown = me
        ? msg.text
        : (msg.done ? (msg.fromEngine ? msg.text : t(msg.text)) : msg.shown);
    final width = MediaQuery.sizeOf(context).width;
    return ConstrainedBox(
      constraints: BoxConstraints(maxWidth: maxW),
      child: IntrinsicWidth(
        child: Container(
          padding: const EdgeInsets.all(16).copyWith(top: 14, bottom: 14),
          decoration: BoxDecoration(
            color: me ? p.accentSoft : p.surface,
            border: Border.all(color: me ? p.accent : p.line),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            mainAxisSize: MainAxisSize.min,
            children: [
              Padding(
                padding: const EdgeInsets.only(bottom: 6),
                child: Mono(
                  me ? t('Moi') : t('Assistant'),
                  color: me ? p.accent : p.dim,
                  height: null,
                ),
              ),
              if (shown.isNotEmpty)
                Text.rich(
                  TextSpan(
                    style: base,
                    children: me
                        ? [TextSpan(text: shown)]
                        : chatSpans(shown, base, p),
                  ),
                )
              else if (!msg.done)
                const _Typing(),
              if (!me && msg.note != null)
                Padding(
                  padding: const EdgeInsets.only(top: 8),
                  child: Text(
                    msg.note!,
                    style: Fonts.sans(size: 13, color: p.dim),
                  ),
                ),
              if (!me && msg.done && msg.draft != null) ...[
                Container(
                  width: double.infinity,
                  margin: const EdgeInsets.only(top: 12),
                  padding: const EdgeInsets.symmetric(
                    horizontal: 14,
                    vertical: 12,
                  ),
                  decoration: BoxDecoration(
                    color: p.surface2,
                    border: Border(left: BorderSide(color: p.accent, width: 2)),
                  ),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      for (var i = 0; i < msg.draft!.length; i++)
                        Padding(
                          padding: EdgeInsets.only(
                            bottom: i == msg.draft!.length - 1 ? 0 : 8,
                          ),
                          child: Text(
                            msg.draft![i],
                            style: Fonts.sans(
                              size: 14.5,
                              color: p.text,
                              height: 1.5,
                            ),
                          ),
                        ),
                    ],
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.only(top: 12),
                  child: Wrap(
                    children: [
                      Btn(
                        icon: 'reply',
                        label: t('Utiliser comme réponse'),
                        go: true,
                        onTap: () => s.useDraft(idx),
                      ),
                    ],
                  ),
                ),
              ],
              if (!me &&
                  msg.done &&
                  msg.fromEngine &&
                  !msg.failed &&
                  msg.text.isNotEmpty &&
                  s.byId(s.open) != null)
                Padding(
                  padding: const EdgeInsets.only(top: 12),
                  child: Wrap(
                    children: [
                      Btn(
                        key: const ValueKey('insert-in-reply'),
                        icon: 'reply',
                        label: t('Insérer dans une réponse'),
                        go: true,
                        onTap: () => s.useDraft(idx),
                      ),
                    ],
                  ),
                ),
              if (!me && msg.done && msg.refs != null && msg.refs!.isNotEmpty)
                Padding(
                  padding: const EdgeInsets.only(top: 12),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: [
                      for (final id in msg.refs!)
                        if (s.byId(id) != null) _Ref(s.byId(id)!, width),
                    ],
                  ),
                ),
            ],
          ),
        ),
      ),
    );
  }
}

class _Ref extends StatelessWidget {
  final dynamic m;
  final double w;
  const _Ref(this.m, this.w);
  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final s = context.read<MailStore>();
    return Padding(
      padding: const EdgeInsets.only(bottom: 6),
      child: Hov(
        onTap: () => s.openRef(m.id),
        builder: (_, h) => Container(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
          decoration: BoxDecoration(
            color: p.bg,
            border: Border.all(color: h ? p.accent : p.line),
          ),
          child: Row(
            children: [
              Avatar(m.from.name, size: 30, font: 10),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      m.subject,
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      style: Fonts.sans(
                        size: 14,
                        w: FontWeight.w500,
                        color: p.text,
                        height: 1.3,
                      ),
                    ),
                    Text(
                      '${m.from.name} · ${rowTime(m.date)}',
                      style: Fonts.sans(size: 12, color: p.dim),
                    ),
                  ],
                ),
              ),
              Transform.rotate(
                angle: 3.14159,
                child: Ico('arrowl', size: 14, color: p.dim),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _Typing extends StatefulWidget {
  const _Typing();
  @override
  State<_Typing> createState() => _TypingState();
}

class _TypingState extends State<_Typing> with SingleTickerProviderStateMixin {
  late final AnimationController ac = AnimationController(
    vsync: this,
    duration: const Duration(seconds: 1),
  )..repeat();
  @override
  void dispose() {
    ac.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: AnimatedBuilder(
        animation: ac,
        builder: (_, _) => Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            for (var i = 0; i < 3; i++) ...[
              if (i > 0) const SizedBox(width: 4),
              Opacity(
                opacity: _op((ac.value - i * .15) % 1),
                child: Container(width: 6, height: 6, color: p.dim),
              ),
            ],
          ],
        ),
      ),
    );
  }

  double _op(double t) =>
      t < .5 ? 1 - (t / .5) * .8 : .2 + ((t - .5) / .5) * .8;
}
