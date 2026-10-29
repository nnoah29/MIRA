import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../data/models.dart';
import '../data/rule_models.dart';
import '../state/local_labels.dart';
import '../state/store.dart';
import '../theme/app_theme.dart';
import '../theme/icons.dart';
import '../theme/wall_source.dart';
import 'about_settings.dart';
import 'body_renderer.dart';
import 'accounts_settings.dart';
import 'contacts_settings.dart';
import 'ai_settings.dart';
import 'image_rules_section.dart';
import 'labels_settings.dart';
import 'templates_settings.dart';
import 'maintenance_settings.dart';
import 'notification_settings.dart';
import 'rules_settings.dart';
import 'widgets.dart';
import '../i18n/i18n.dart';

/// Couleurs principales proposées (valeurs du thème sombre ; le thème clair en dérive).
List<(String, String)> get accentPresets => <(String, String)>[
  (t('Orange'), 'e8542b'),
  (t('Rouge'), 'e5484d'),
  (t('Rose'), 'e0457b'),
  (t('Violet'), '8e6bf0'),
  (t('Bleu'), '3b82f6'),
  (t('Cyan'), '22b8cf'),
  (t('Vert'), '2fb36d'),
  (t('Jaune'), 'e5b100'),
];

List<String> _tabs(bool engine) => [
  t('Apparence'),
  t('Messagerie'),
  t('Notifications'),
  t('Comptes'),
  t('Contacts'),
  if (engine) t('Libellés'),
  if (engine) t('Règles'),
  if (engine) t('Modèles'),
  if (engine) t('Intelligence artificielle'),
  if (engine) t('Stockage'),
  t('À propos'),
];

class SettingsPanel extends StatefulWidget {
  const SettingsPanel({super.key});
  @override
  State<SettingsPanel> createState() => _SettingsPanelState();
}

class _SettingsPanelState extends State<SettingsPanel> {
  int tab = 0;
  final _pageCtl = PageController();
  final _tabKeys = <int, GlobalKey>{};
  final ruleDraft = ValueNotifier<Rule?>(null);
  late final TextEditingController sigCtl;

  @override
  void initState() {
    super.initState();
    sigCtl = TextEditingController(text: context.read<MailStore>().sig);
  }

  @override
  void dispose() {
    sigCtl.dispose();
    ruleDraft.dispose();
    _pageCtl.dispose();
    super.dispose();
  }

  /// Change d'onglet : la page glisse si l'onglet est voisin, saute sinon.
  void _goTo(int i) {
    final far = (i - tab).abs() > 1;
    setState(() => tab = i);
    WidgetsBinding.instance.addPostFrameCallback((_) => _revealTab(i));
    if (!_pageCtl.hasClients) return;
    if (far) return _pageCtl.jumpToPage(i);
    _pageCtl.animateToPage(
      i,
      duration: const Duration(milliseconds: 280),
      curve: Curves.easeOutCubic,
    );
  }

  /// Suit le doigt : met à jour l'onglet actif quand une page s'est installée.
  void _pageChanged(int i) {
    if (i == tab) return;
    setState(() => tab = i);
    WidgetsBinding.instance.addPostFrameCallback((_) => _revealTab(i));
  }

  /// Recale la page affichée si l'onglet a changé sans geste (ex. ajout de compte).
  void _syncPage() {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted || !_pageCtl.hasClients) return;
      final page = _pageCtl.page;
      if (page == null || page == tab.toDouble()) return;
      if (_pageCtl.position.isScrollingNotifier.value) return;
      _pageCtl.jumpToPage(tab);
    });
  }

  /// Pages d'onglets qui suivent le doigt (mobile).
  Widget _pagedContent(int count, double hp, Widget Function(int) contentFor) {
    _syncPage();
    return PageView.builder(
      controller: _pageCtl,
      itemCount: count,
      onPageChanged: _pageChanged,
      itemBuilder: (_, i) => ScrollArea(
        key: ValueKey('settings-page-$i'),
        padding: EdgeInsets.fromLTRB(hp, 0, hp, 40),
        child: contentFor(i),
      ),
    );
  }

  /// Fait défiler la barre d'onglets pour montrer l'onglet choisi.
  void _revealTab(int i) {
    final ctx = _tabKeys[i]?.currentContext;
    if (ctx == null) return;
    Scrollable.ensureVisible(
      ctx,
      alignment: .5,
      duration: const Duration(milliseconds: 200),
    );
  }

  @override
  Widget build(BuildContext context) {
    final s = context.watch<MailStore>();
    final p = context.pal;
    final w = MediaQuery.sizeOf(context).width;
    final narrow = w <= 1100;
    final hp = narrow ? 16.0 : 32.0;

    Widget seg(String k, List<(String, String)> opts, String cur) => Container(
      decoration: BoxDecoration(border: Border.all(color: p.line)),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          for (var i = 0; i < opts.length; i++)
            Hov(
              onTap: () => s.setSetting(k, opts[i].$1),
              builder: (_, h) {
                final on = cur == opts[i].$1;
                return AnimatedContainer(
                  duration: const Duration(milliseconds: 130),
                  padding: const EdgeInsets.symmetric(
                    horizontal: 14,
                    vertical: 8,
                  ),
                  decoration: BoxDecoration(
                    color: on ? p.accentSoft : null,
                    border: i > 0
                        ? Border(left: BorderSide(color: p.line))
                        : null,
                  ),
                  child: Mono(
                    opts[i].$2,
                    ls: .12,
                    color: on ? p.accent : (h ? p.text : p.muted),
                    height: null,
                  ),
                );
              },
            ),
        ],
      ),
    );

    Widget head(String t, String d) => Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          t,
          style: Fonts.sans(size: 15, w: FontWeight.w500, color: p.text),
        ),
        Text(d, style: Fonts.sans(size: 13.5, color: p.dim)),
      ],
    );

    Widget srow(String t, String d, Widget ctl, {bool last = false}) =>
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 16),
          decoration: BoxDecoration(
            border: last ? null : Border(bottom: BorderSide(color: p.lineSoft)),
          ),
          child: Wrap(
            alignment: WrapAlignment.spaceBetween,
            crossAxisAlignment: WrapCrossAlignment.center,
            runSpacing: 12,
            spacing: 24,
            children: [
              ConstrainedBox(
                constraints: const BoxConstraints(maxWidth: 480),
                child: head(t, d),
              ),
              ctl,
            ],
          ),
        );

    Widget box(List<Widget> kids) => Container(
      margin: const EdgeInsets.only(top: 24),
      decoration: BoxDecoration(
        color: p.surface,
        border: Border.all(color: p.line),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: kids,
      ),
    );

    if (s.wantsAddAccount && s.source != null) tab = 3;
    final aboutTab = _tabs(s.source != null).length - 1;
    Widget contentFor(int index) => index == aboutTab
        ? box([const AboutSettings()])
        : switch (index) {
            0 => box([
              srow(
                t('Langue de l’application'),
                t(
                  'Choisis la langue de l’interface et des messages fictifs de la démonstration.',
                ),
                Container(
                  decoration: BoxDecoration(border: Border.all(color: p.line)),
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      for (final (i, l) in AppLang.values.indexed)
                        Hov(
                          onTap: () => s.setLang(l),
                          builder: (_, h) {
                            final on = s.lang == l;
                            return AnimatedContainer(
                              duration: const Duration(milliseconds: 130),
                              padding: const EdgeInsets.symmetric(
                                horizontal: 14,
                                vertical: 8,
                              ),
                              decoration: BoxDecoration(
                                color: on ? p.accentSoft : null,
                                border: i > 0
                                    ? Border(left: BorderSide(color: p.line))
                                    : null,
                              ),
                              child: Mono(
                                l.label,
                                ls: .12,
                                color: on ? p.accent : (h ? p.text : p.muted),
                                height: null,
                              ),
                            );
                          },
                        ),
                    ],
                  ),
                ),
              ),
              if (wallSupported)
                Container(
                  padding: const EdgeInsets.symmetric(
                    horizontal: 20,
                    vertical: 16,
                  ),
                  decoration: BoxDecoration(
                    border: Border(bottom: BorderSide(color: p.lineSoft)),
                  ),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: [
                      Wrap(
                        alignment: WrapAlignment.spaceBetween,
                        crossAxisAlignment: WrapCrossAlignment.center,
                        runSpacing: 12,
                        spacing: 24,
                        children: [
                          ConstrainedBox(
                            constraints: BoxConstraints(maxWidth: 520),
                            child: head(
                              t('Couleurs du fond d’écran'),
                              s.wallAvailable
                                  ? t(
                                      'Fond, texte et accent suivent ton fond d’écran en direct (ShellArie).',
                                    )
                                  : t(
                                      'Aucun fichier de couleurs trouvé (/tmp/qs_colors.json) : thème MIRA par défaut.',
                                    ),
                            ),
                          ),
                          seg('follow', [
                            ('1', t('Suivre')),
                            ('0', t('Ignorer')),
                          ], s.followWallpaper ? '1' : '0'),
                        ],
                      ),
                      if (s.wallAvailable && s.followWallpaper) ...[
                        const SizedBox(height: 14),
                        Row(
                          children: [
                            for (final (n, c) in [
                              (t('Fond'), p.bg),
                              (t('Surface'), p.surface2),
                              (t('Texte'), p.text),
                              (t('Accent'), p.accent),
                            ])
                              Padding(
                                padding: const EdgeInsets.only(right: 14),
                                child: Row(
                                  mainAxisSize: MainAxisSize.min,
                                  children: [
                                    Container(
                                      width: 22,
                                      height: 22,
                                      decoration: BoxDecoration(
                                        color: c,
                                        border: Border.all(color: p.line),
                                      ),
                                    ),
                                    SizedBox(width: 8),
                                    Mono(n, size: 10.5, ls: .1, height: null),
                                  ],
                                ),
                              ),
                          ],
                        ),
                      ],
                    ],
                  ),
                ),
              Opacity(
                opacity: s.wallPal != null ? .4 : 1,
                child: IgnorePointer(
                  ignoring: s.wallPal != null,
                  child: srow(
                    t('Thème'),
                    s.wallPal != null
                        ? t(
                            'Piloté par le fond d’écran (clair ou sombre selon sa luminosité).',
                          )
                        : t('Sombre ou clair, selon ton environnement.'),
                    seg('theme', [
                      ('dark', t('Sombre')),
                      ('light', t('Clair')),
                    ], s.theme),
                  ),
                ),
              ),
              Container(
                padding: EdgeInsets.symmetric(horizontal: 20, vertical: 16),
                decoration: BoxDecoration(
                  border: Border(bottom: BorderSide(color: p.lineSoft)),
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    head(
                      t('Couleur principale'),
                      t(
                        'Remplace l’orange : boutons, accents, sélection et libellés actifs.',
                      ),
                    ),
                    SizedBox(height: 14),
                    _AccentPicker(),
                  ],
                ),
              ),
              srow(
                t('Densité de la liste'),
                t(
                  'Confortable : deux lignes d’aperçu sous l’objet. Compacte : plus de messages à l’écran, sans aperçu.',
                ),
                seg('dense', [
                  ('0', t('Confortable')),
                  ('1', t('Compacte')),
                ], s.dense ? '1' : '0'),
                last: !webViewEnabled,
              ),
              if (webViewEnabled)
                srow(
                  t('Affichage du HTML'),
                  t(
                    'Auto : les messages simples en widgets, les autres dans la vue web. Vue web : fidèle à un navigateur. Widgets : intégré à l’app, plus léger, moins fidèle.',
                  ),
                  seg('htmlMode', [
                    ('auto', t('Auto')),
                    ('web', t('Vue web')),
                    ('widgets', t('Widgets')),
                  ], s.htmlMode.code),
                  last: true,
                ),
            ]),
            1 => box([
              if (s.source == null)
                srow(
                  t('Nouveaux messages simulés'),
                  t(
                    'Reçois de faux mails à intervalles réguliers pour la démo.',
                  ),
                  seg('live', [
                    ('1', t('Activé')),
                    ('0', t('Désactivé')),
                  ], s.live ? '1' : '0'),
                ),
              if (s.source != null)
                srow(
                  t('Fréquence de relevé'),
                  t(
                    'Nouveaux messages de la Réception : instantané (IDLE). Les autres changements sont relevés à cette fréquence, appliquée tout de suite. Plus court = plus de trafic, certains fournisseurs peuvent limiter le compte.',
                  ),
                  seg('poll', [
                    ('', t('Auto')),
                    ('30', '30 s'),
                    ('60', '1 min'),
                    ('120', '2 min'),
                    ('300', '5 min'),
                  ], s.pollChoice),
                ),
              srow(
                t('Hibernation'),
                t(
                  'Sans le focus depuis ce délai, MIRA libère l’affichage (corps des messages, images, caches) ; la réception, les règles et les notifications continuent.',
                ),
                seg('hibernate', [
                  ('0', t('Jamais')),
                  ('5', '5 min'),
                  ('10', '10 min'),
                  ('30', '30 min'),
                ], '${s.hibernateMinutes}'),
              ),
              const ImageRulesSection(),
              Divider(height: 1, color: p.lineSoft),
              Padding(
                padding: const EdgeInsets.fromLTRB(20, 16, 20, 16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    head(
                      t('Signature'),
                      t('Ajoutée à tes réponses et transferts.'),
                    ),
                    const SizedBox(height: 12),
                    _SigField(ctl: sigCtl, onChanged: s.setSig),
                  ],
                ),
              ),
            ]),
            2 => box([const NotificationSettings()]),
            4 => box([const ContactsSettings()]),
            5 when s.source?.labelSettingsAdmin != null => box([
              const LabelsSettings(),
            ]),
            6 when s.source?.ruleAdmin != null => box([
              RulesSettings(
                draft: ruleDraft,
                onApplied: s.rulesApplied,
                admin: s.source!.ruleAdmin!,
                accounts: {for (final a in s.accounts) a.id: a.email},
                changes: s.source!.ruleChanges,
                templateAdmin: s.source!.templateAdmin,
                labels: [
                  if (s.localLabelsOnEngine)
                    for (final e in s.localLabels.defs.entries)
                      RuleChoice(
                        e.key.substring(localLabelPrefix.length),
                        e.value.n,
                      ),
                ],
              ),
            ]),
            7 when s.source?.templateAdmin != null => box([
              TemplatesSettings(
                admin: s.source!.templateAdmin!,
                changes: s.source!.templateChanges,
              ),
            ]),
            8 when s.aiAdmin != null => box([AiSettings(admin: s.aiAdmin!)]),
            9 when s.source != null => box([const MaintenanceSettings()]),
            _ when s.source != null => box([const AccountsSettings()]),
            _ => box([
              srow(me.name, me.email, const SizedBox.shrink()),
              srow(
                t('Démo'),
                t('Remet tous les messages et réglages de départ.'),
                Btn(
                  icon: 'reply',
                  label: t('Réinitialiser'),
                  onTap: () => s.setSetting('reset', '1'),
                ),
                last: true,
              ),
            ]),
          };

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Padding(
          padding: EdgeInsets.fromLTRB(hp, 24, hp, 12),
          child: Row(
            children: [
              Text(
                t('PARAMÈTRES'),
                style: Fonts.display(size: 24, ls: .02, color: p.text),
              ),
              const Spacer(),
              IBtn('x', tip: t('Fermer'), onTap: s.closePane),
            ],
          ),
        ),
        // Onglets
        Container(
          margin: EdgeInsets.symmetric(horizontal: hp),
          decoration: BoxDecoration(
            border: Border(bottom: BorderSide(color: p.line)),
          ),
          child: SingleChildScrollView(
            scrollDirection: Axis.horizontal,
            child: Row(
              children: [
                for (var i = 0; i < _tabs(s.source != null).length; i++)
                  KeyedSubtree(
                    key: _tabKeys.putIfAbsent(i, GlobalKey.new),
                    child: Hov(
                      key: ValueKey('settings-tab-$i'),
                      onTap: () => _goTo(i),
                      builder: (_, h) {
                        final on = tab == i;
                        return AnimatedContainer(
                          duration: const Duration(milliseconds: 130),
                          padding: const EdgeInsets.fromLTRB(2, 10, 2, 11),
                          margin: const EdgeInsets.only(right: 28),
                          decoration: BoxDecoration(
                            border: Border(
                              bottom: BorderSide(
                                color: on ? p.accent : Colors.transparent,
                                width: 2,
                              ),
                            ),
                          ),
                          child: Mono(
                            _tabs(s.source != null)[i],
                            ls: .14,
                            color: on ? p.accent : (h ? p.text : p.muted),
                            height: null,
                          ),
                        );
                      },
                    ),
                  ),
              ],
            ),
          ),
        ),
        Expanded(
          child: w <= 760
              ? _pagedContent(_tabs(s.source != null).length, hp, contentFor)
              : ScrollArea(
                  padding: EdgeInsets.fromLTRB(hp, 0, hp, 40),
                  child: FadeSlideIn(
                    key: ValueKey(tab),
                    from: const Offset(0, 8),
                    duration: const Duration(milliseconds: 220),
                    child: contentFor(tab),
                  ),
                ),
        ),
      ],
    );
  }
}

/// Pastilles de couleurs + champ hexadécimal pour une couleur personnalisée.
class _AccentPicker extends StatefulWidget {
  const _AccentPicker();
  @override
  State<_AccentPicker> createState() => _AccentPickerState();
}

class _AccentPickerState extends State<_AccentPicker> {
  late final TextEditingController hex;

  @override
  void initState() {
    super.initState();
    hex = TextEditingController(
      text: context.read<MailStore>().accentHex ?? '',
    );
  }

  @override
  void dispose() {
    hex.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final s = context.watch<MailStore>();
    final p = context.pal;
    final auto = s.wallPal != null;
    final cur = s.accentHex ?? (auto ? 'auto' : accentPresets.first.$2);
    final isPreset = cur == 'auto' || accentPresets.any((e) => e.$2 == cur);
    final typed = hex.text.trim().replaceAll('#', '');
    final bad =
        typed.isNotEmpty && !RegExp(r'^[0-9a-fA-F]{6}$').hasMatch(typed);

    return Wrap(
      spacing: 10,
      runSpacing: 12,
      crossAxisAlignment: WrapCrossAlignment.center,
      children: [
        if (auto)
          Hov(
            tip: t('Fond d’écran'),
            onTap: () {
              hex.text = '';
              s.setAccent(null);
            },
            builder: (_, hover) {
              final c = s.wallPal!.accent;
              final on = cur == 'auto';
              return AnimatedContainer(
                duration: const Duration(milliseconds: 130),
                width: 34,
                height: 34,
                padding: const EdgeInsets.all(3),
                decoration: BoxDecoration(
                  border: Border.all(
                    color: on ? p.text : (hover ? p.dim : p.line),
                  ),
                ),
                child: Container(
                  color: c,
                  alignment: Alignment.center,
                  child: Ico(
                    'spark',
                    size: 14,
                    stroke: 2,
                    color: c.computeLuminance() > .45
                        ? const Color(0xFF1b1b1b)
                        : Colors.white,
                  ),
                ),
              );
            },
          ),
        for (final (name, h) in accentPresets)
          Hov(
            tip: name,
            onTap: () {
              final stock = h == accentPresets.first.$2 && !auto;
              hex.text = stock ? '' : h;
              s.setAccent(stock ? null : h);
            },
            builder: (_, hover) {
              final c = Color(0xFF000000 | int.parse(h, radix: 16));
              final on = cur == h;
              return AnimatedContainer(
                duration: const Duration(milliseconds: 130),
                width: 34,
                height: 34,
                padding: const EdgeInsets.all(3),
                decoration: BoxDecoration(
                  border: Border.all(
                    color: on ? p.text : (hover ? p.dim : p.line),
                  ),
                ),
                child: Container(
                  color: c,
                  alignment: Alignment.center,
                  child: on
                      ? Ico(
                          'check',
                          size: 14,
                          stroke: 2.6,
                          color: c.computeLuminance() > .45
                              ? const Color(0xFF1b1b1b)
                              : Colors.white,
                        )
                      : null,
                ),
              );
            },
          ),
        const SizedBox(width: 6),
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 7),
          decoration: BoxDecoration(
            color: p.bg,
            border: Border.all(
              color: bad ? p.accent : (!isPreset ? p.text : p.line),
            ),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              if (!isPreset)
                Container(
                  width: 14,
                  height: 14,
                  margin: const EdgeInsets.only(right: 8),
                  color: s.accentColor,
                ),
              Text('#', style: Fonts.mono(size: 13, color: p.dim, ls: 0)),
              SizedBox(
                width: 104,
                child: TextField(
                  controller: hex,
                  maxLength: 7,
                  onChanged: (v) {
                    final t = v.trim().replaceAll('#', '');
                    if (RegExp(r'^[0-9a-fA-F]{6}$').hasMatch(t)) s.setAccent(t);
                    setState(() {});
                  },
                  cursorColor: p.accent,
                  cursorWidth: 1,
                  style: Fonts.mono(size: 13, color: p.text, ls: 0),
                  decoration: InputDecoration(
                    isDense: true,
                    counterText: '',
                    border: InputBorder.none,
                    contentPadding: const EdgeInsets.only(left: 2),
                    hintText: t('personnalisée'),
                    hintStyle: Fonts.mono(size: 11, color: p.dim, ls: 0),
                  ),
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }
}

class _SigField extends StatefulWidget {
  final TextEditingController ctl;
  final ValueChanged<String> onChanged;
  const _SigField({required this.ctl, required this.onChanged});
  @override
  State<_SigField> createState() => _SigFieldState();
}

class _SigFieldState extends State<_SigField> {
  final fn = FocusNode();
  @override
  void initState() {
    super.initState();
    fn.addListener(() => setState(() {}));
  }

  @override
  void dispose() {
    fn.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
      decoration: BoxDecoration(
        color: p.bg,
        border: Border.all(color: fn.hasFocus ? p.accent : p.line),
      ),
      child: TextField(
        controller: widget.ctl,
        focusNode: fn,
        minLines: 3,
        maxLines: 6,
        onChanged: widget.onChanged,
        cursorColor: p.accent,
        cursorWidth: 1,
        style: Fonts.sans(size: 15, color: p.text),
        decoration: const InputDecoration(
          isDense: true,
          border: InputBorder.none,
          contentPadding: EdgeInsets.zero,
        ),
      ),
    );
  }
}
