import 'dart:async';

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../engine/account_setup.dart';
import '../engine/engine_client.dart';
import '../i18n/i18n.dart';
import '../state/store.dart';
import '../theme/app_theme.dart';
import 'account_form_widgets.dart';
import 'html_body.dart';
import 'widgets.dart';

final _addressRe = RegExp(r'^[^\s@<>,;]+@[^\s@<>,;]+\.[^\s@<>,;]+$');

/// Panneau « Ajouter un compte » : fournisseurs pris en charge, identité, authentification, serveurs détaillés.
class AddAccountPanel extends StatefulWidget {
  /// Dans les réglages : sans en-tête ni défilement propre, `onAdded` reçoit le compte créé.
  final bool embedded;
  final void Function(String id)? onAdded;
  const AddAccountPanel({super.key, this.embedded = false, this.onAdded});

  @override
  State<AddAccountPanel> createState() => _AddAccountPanelState();
}

class _AddAccountPanelState extends State<AddAccountPanel> {
  late final AccountSetup setup;
  List<ProviderInfo>? providers;
  String? loadError;
  ProviderInfo? provider;
  final name = TextEditingController();
  final address = TextEditingController();
  final password = TextEditingController();
  final imapHost = TextEditingController();
  final imapPort = TextEditingController(text: '993');
  final smtpHost = TextEditingController();
  final smtpPort = TextEditingController(text: '587');
  String method = 'password', imapSec = 'tls', smtpSec = 'start_tls';
  bool serversTouched = false, busy = false, showPassword = false;
  String? error, info, oauthFlow, oauthUrl;
  Endpoint? detectedImap, detectedSmtp;
  Timer? _detect;

  @override
  void initState() {
    super.initState();
    setup = context.read<MailStore>().source!.setup!;
    unawaited(_loadProviders());
  }

  @override
  void dispose() {
    _detect?.cancel();
    final flow = oauthFlow;
    if (flow != null) unawaited(setup.cancelOAuth(flow).catchError((_) {}));
    for (final c in [
      name,
      address,
      password,
      imapHost,
      imapPort,
      smtpHost,
      smtpPort,
    ]) {
      c.dispose();
    }
    super.dispose();
  }

  Future<void> _loadProviders() async {
    try {
      final list = await setup.providers();
      if (mounted) setState(() => providers = list);
    } catch (e) {
      if (mounted) setState(() => loadError = _message(e));
    }
  }

  String _message(Object e) =>
      e is EngineException ? e.message : t('Erreur inattendue : {0}', ['$e']);

  void _selectProvider(ProviderInfo p) {
    setState(() {
      provider = p;
      method = p.methods.isEmpty ? 'password' : p.methods.first;
      error = info = null;
    });
    unawaited(_prepare());
  }

  void _onAddress(String _) {
    setState(() => error = info = null);
    _detect?.cancel();
    _detect = Timer(const Duration(milliseconds: 450), _prepare);
  }

  /// Demande au moteur le fournisseur et les serveurs de cette adresse (silencieux en cas d'échec).
  Future<void> _prepare() async {
    final a = address.text.trim();
    if (!_addressRe.hasMatch(a)) return;
    try {
      final prep = await setup.prepare(a, providerId: provider?.id);
      if (!mounted) return;
      setState(() {
        provider ??=
            providers?.where((p) => p.id == prep.provider.id).firstOrNull ??
            prep.provider;
        if (!provider!.methods.contains(method)) {
          method = provider!.methods.firstOrNull ?? 'password';
        }
        detectedImap = prep.imap;
        detectedSmtp = prep.smtp;
        if (!serversTouched) _fillServers(prep.imap, prep.smtp);
      });
    } on EngineException {
      return;
    }
  }

  void _fillServers(Endpoint i, Endpoint s) {
    imapHost.text = i.host;
    imapPort.text = '${i.port}';
    imapSec = i.security;
    smtpHost.text = s.host;
    smtpPort.text = '${s.port}';
    smtpSec = s.security;
  }

  Endpoint? _endpoint(
    TextEditingController h,
    TextEditingController p,
    String sec,
    String label,
  ) {
    final host = h.text.trim();
    final port = int.tryParse(p.text.trim());
    if (host.isEmpty || host.contains(RegExp(r'\s'))) {
      error = t('Serveur {0} : hôte invalide', [label]);
      return null;
    }
    if (port == null || port < 1 || port > 65535) {
      error = t('Serveur {0} : port invalide (1 à 65535)', [label]);
      return null;
    }
    return Endpoint(host, port, sec);
  }

  /// Compte saisi, ou `null` avec `error` renseigné.
  NewAccount? _build({required bool needSecret}) {
    final p = provider;
    final a = address.text.trim();
    if (p == null) {
      error = t('Choisis un fournisseur ou saisis ton adresse.');
    } else if (!_addressRe.hasMatch(a)) {
      error = t('Adresse e-mail invalide.');
    } else if (needSecret && password.text.isEmpty) {
      error = t('Saisis le mot de passe.');
    } else {
      Endpoint? imap, smtp;
      if (p.isManual) {
        imap = _endpoint(imapHost, imapPort, imapSec, 'IMAP');
        smtp = _endpoint(smtpHost, smtpPort, smtpSec, 'SMTP');
        if (imap == null || smtp == null) return null;
      }
      return NewAccount(
        address: a,
        displayName: name.text.trim(),
        providerId: p.id,
        method: method,
        imap: imap,
        smtp: smtp,
      );
    }
    return null;
  }

  Future<void> _run(Future<void> Function() job) async {
    setState(() {
      busy = true;
      error = info = null;
    });
    try {
      await job();
    } catch (e) {
      if (mounted) setState(() => error = _message(e));
    } finally {
      if (mounted) setState(() => busy = false);
    }
  }

  String _key(NewAccount a) =>
      'flutter-add-${Object.hash(a.address, a.providerId, a.method, a.imap?.host, a.imap?.port, a.smtp?.host, a.smtp?.port)}';

  Future<void> _test() async {
    final a = _build(needSecret: true);
    if (a == null) return setState(() {});
    await _run(() async {
      await setup.test(a, password.text);
      if (mounted) {
        setState(
          () =>
              info = t('Connexion réussie : adresse et mot de passe acceptés.'),
        );
      }
    });
  }

  Future<void> _add() async {
    final a = _build(needSecret: true);
    if (a == null) return setState(() {});
    await _run(() async {
      final secret = password.text;
      final id = await setup.add(a, secret, _key(a));
      password.clear();
      await _finish(id);
    });
  }

  Future<void> _oauth() async {
    final a = _build(needSecret: false);
    if (a == null) return setState(() {});
    await _run(() async {
      final started = await setup.startOAuth(a.address, a.providerId);
      setState(() {
        oauthFlow = started.flow;
        oauthUrl = started.url;
      });
      final opened = await openLink(started.url);
      if (!opened && mounted) {
        setState(
          () => info = t('Ouvre cette adresse dans ton navigateur : {0}', [
            started.url,
          ]),
        );
      }
      await setup.completeOAuth(started.flow);
      final id = await setup.addOAuth(a, started.flow, _key(a));
      oauthFlow = null;
      await _finish(id);
    });
    if (mounted) setState(() => oauthFlow = null);
  }

  Future<void> _finish(String id) async {
    final store = context.read<MailStore>();
    unawaited(setup.startSync(id).catchError((_) {}));
    if (widget.embedded) {
      await store.accountsChanged();
      widget.onAdded?.call(id);
    } else {
      await store.accountAdded(id);
    }
    store.toast(t('Compte ajouté : {0}', [address.text.trim()]));
  }

  Future<void> _cancelOAuth() async {
    final flow = oauthFlow;
    if (flow == null) return;
    setState(() => oauthFlow = null);
    try {
      await setup.cancelOAuth(flow);
    } catch (_) {
      return;
    }
  }

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final s = context.read<MailStore>();
    final narrow = MediaQuery.sizeOf(context).width <= 1100;
    final hp = narrow ? 16.0 : 32.0;
    if (widget.embedded) return _sections(p);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Padding(
          padding: EdgeInsets.fromLTRB(hp, 24, hp, 12),
          child: Row(
            children: [
              Text(
                t('AJOUTER UN COMPTE'),
                style: Fonts.display(size: 24, ls: .02, color: p.text),
              ),
              const Spacer(),
              IBtn('x', tip: t('Fermer'), onTap: s.closePane),
            ],
          ),
        ),
        Expanded(
          child: ScrollArea(
            padding: EdgeInsets.fromLTRB(hp, 0, hp, 40),
            child: _sections(p),
          ),
        ),
      ],
    );
  }

  Widget _sections(Pal p) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      _providerSection(),
      if (provider != null) ...[
        _identitySection(),
        _authSection(),
        if (provider!.isManual) _serversSection(),
        _actions(p),
      ],
    ],
  );

  Widget _providerSection() => FormSection(
    title: t('Fournisseur'),
    hint: t(
      'Choisis ton fournisseur, ou saisis ton adresse plus bas : il sera détecté.',
    ),
    child: loadError != null
        ? Text(loadError!, style: TextStyle(color: context.pal.accent))
        : providers == null
        ? const SizedBox(
            height: 40,
            child: Center(child: CircularProgressIndicator(strokeWidth: 2)),
          )
        : Wrap(
            spacing: 12,
            runSpacing: 12,
            children: [
              for (final pr in providers!)
                ProviderCard(
                  key: ValueKey('provider-${pr.id}'),
                  name: pr.isManual ? t('Autre (IMAP / SMTP)') : pr.name,
                  subtitle: pr.isManual
                      ? t('Serveurs à la main')
                      : pr.domains.take(2).join(', '),
                  selected: provider?.id == pr.id,
                  onTap: () => _selectProvider(pr),
                ),
            ],
          ),
  );

  Widget _identitySection() => FormSection(
    title: t('Identité'),
    child: Wrap(
      spacing: 14,
      runSpacing: 12,
      children: [
        FormField2(
          fieldKey: const ValueKey('field-name'),
          label: t('NOM AFFICHÉ'),
          controller: name,
          width: 280,
          hint: t('Ton nom'),
        ),
        FormField2(
          fieldKey: const ValueKey('field-address'),
          label: t('ADRESSE E-MAIL'),
          controller: address,
          width: 340,
          keyboard: TextInputType.emailAddress,
          hint: 'nom@exemple.com',
          onChanged: _onAddress,
        ),
      ],
    ),
  );

  Widget _authSection() {
    final p = context.pal;
    final methods = provider!.methods.isEmpty
        ? ['password']
        : provider!.methods;
    return FormSection(
      title: t('Authentification'),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          if (methods.length > 1)
            Padding(
              padding: const EdgeInsets.only(bottom: 14),
              child: Segments<String>(
                keyPrefix: 'method',
                options: [for (final m in methods) (m, methodLabel(m))],
                value: method,
                onChanged: (m) => setState(() => method = m),
              ),
            ),
          if (method == 'oauth2')
            Text(
              t(
                'Une page de {0} s’ouvre dans ton navigateur pour autoriser MIRA ; aucun mot de passe n’est saisi ici.',
                [provider!.name],
              ),
              style: Fonts.sans(size: 13.5, color: p.dim),
            )
          else
            FormField2(
              fieldKey: const ValueKey('field-password'),
              label: method == 'app_password'
                  ? t('MOT DE PASSE D’APPLICATION')
                  : t('MOT DE PASSE'),
              controller: password,
              secret: !showPassword,
              width: 340,
              trailing: IBtn(
                showPassword ? 'x' : 'eye',
                tip: showPassword ? t('Masquer') : t('Afficher'),
                onTap: () => setState(() => showPassword = !showPassword),
              ),
            ),
        ],
      ),
    );
  }

  Widget _serversSection() => FormSection(
    title: t('Serveurs'),
    hint: t(
      'Réglages détaillés du protocole : IMAP pour recevoir, SMTP pour envoyer.',
    ),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        ServerFields(
          title: t('RÉCEPTION · IMAP'),
          keyPrefix: 'imap',
          host: imapHost,
          port: imapPort,
          security: imapSec,
          onSecurity: (v) => setState(() {
            imapSec = v;
            serversTouched = true;
            imapPort.text = v == 'tls' ? '993' : '143';
          }),
          onEdited: () => serversTouched = true,
        ),
        const SizedBox(height: 20),
        ServerFields(
          title: t('ENVOI · SMTP'),
          keyPrefix: 'smtp',
          host: smtpHost,
          port: smtpPort,
          security: smtpSec,
          onSecurity: (v) => setState(() {
            smtpSec = v;
            serversTouched = true;
            smtpPort.text = v == 'tls' ? '465' : '587';
          }),
          onEdited: () => serversTouched = true,
        ),
      ],
    ),
  );

  Widget _actions(Pal p) {
    final oauth = method == 'oauth2';
    return Padding(
      padding: const EdgeInsets.only(top: 20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Wrap(
            spacing: 10,
            runSpacing: 10,
            children: [
              if (oauth)
                oauthFlow == null
                    ? Btn(
                        key: const ValueKey('btn-oauth'),
                        icon: 'send',
                        label: t('Se connecter avec {0}', [provider!.name]),
                        go: true,
                        onTap: busy ? null : _oauth,
                      )
                    : Btn(label: t('Annuler'), onTap: _cancelOAuth)
              else ...[
                Btn(
                  key: const ValueKey('btn-add'),
                  icon: 'plus',
                  label: t('Ajouter le compte'),
                  go: true,
                  onTap: busy ? null : _add,
                ),
                Btn(
                  key: const ValueKey('btn-test'),
                  icon: 'reply',
                  label: t('Tester la connexion'),
                  onTap: busy ? null : _test,
                ),
              ],
            ],
          ),
          if (busy)
            Padding(
              padding: const EdgeInsets.only(top: 12),
              child: Text(
                oauthFlow != null
                    ? t('En attente de ton autorisation dans le navigateur…')
                    : t('Connexion en cours…'),
                style: Fonts.sans(size: 14, color: p.muted),
              ),
            ),
          if (error != null)
            Padding(
              padding: const EdgeInsets.only(top: 12),
              child: Text(
                error!,
                key: const ValueKey('form-error'),
                style: Fonts.sans(size: 14, color: p.accent),
              ),
            ),
          if (info != null)
            Padding(
              padding: const EdgeInsets.only(top: 12),
              child: SelectableText(
                info!,
                key: const ValueKey('form-info'),
                style: Fonts.sans(size: 14, color: p.muted),
              ),
            ),
        ],
      ),
    );
  }
}
