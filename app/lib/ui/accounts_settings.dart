import 'dart:async';

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../engine/account_admin.dart';
import '../engine/account_setup.dart';
import '../engine/engine_client.dart';
import '../i18n/engine_errors.dart';
import '../i18n/i18n.dart';
import '../state/store.dart';
import '../theme/app_theme.dart';
import 'account_form_widgets.dart';
import 'account_list_rows.dart';
import 'account_tabs.dart';
import 'add_account_panel.dart';
import 'collapsible_section.dart';
import 'html_body.dart';
import 'identities_box.dart';
import 'pending_actions.dart';
import 'sync_status_line.dart';
import 'synced_mailboxes.dart';
import 'widgets.dart';

String _message(Object e) => engineErrorText(e);

/// Libellé et importance de l'état d'un compte.
(String, bool) stateLabel(AccountDetail a) => switch (a.state) {
  'connected' => (t('Connecté'), false),
  'offline' => (t('Hors ligne'), false),
  'auth_required' => (t('Reconnexion nécessaire'), true),
  'paused' => (t('En pause'), false),
  _ => (t('Erreur'), true),
};

/// Paramètres › Comptes : un onglet par compte, avec ses réglages en dessous.
class AccountsSettings extends StatefulWidget {
  const AccountsSettings({super.key});

  @override
  State<AccountsSettings> createState() => _AccountsSettingsState();
}

class _AccountsSettingsState extends State<AccountsSettings> {
  late final AccountAdmin admin;
  late final AccountSetup setup;
  late final MailStore _store;
  List<AccountDetail>? accounts;
  String? error, selectedId, openId;
  bool adding = false, _mobile = false;

  @override
  void initState() {
    super.initState();
    _store = context.read<MailStore>();
    final src = _store.source!;
    admin = src.admin!;
    setup = src.setup!;
    _store.paneBack = _handleBack;
    unawaited(load());
  }

  @override
  void dispose() {
    if (_store.paneBack == _handleBack) _store.paneBack = null;
    super.dispose();
  }

  /// Retour système sur mobile : de la page d'un compte vers la liste (vrai si traité).
  bool _handleBack() {
    if (!_mobile || !(adding || openId != null)) return false;
    setState(() => adding = false);
    setState(() => openId = null);
    return true;
  }

  Future<void> load() async {
    try {
      final list = await admin.accounts();
      if (mounted) setState(() => accounts = list);
    } catch (e) {
      if (mounted) setState(() => error = _message(e));
    }
  }

  /// Relit le moteur puis met à jour la liste des comptes de l'application.
  Future<void> changed() async {
    await context.read<MailStore>().accountsChanged();
    await load();
  }

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final s = context.watch<MailStore>();
    if (s.wantsAddAccount) {
      adding = true;
      s.wantsAddAccount = false;
    }
    if (error != null) {
      return Padding(
        padding: const EdgeInsets.all(20),
        child: Text(error!, style: Fonts.sans(size: 14, color: p.accent)),
      );
    }
    final list = accounts;
    if (list == null) {
      return const Padding(
        padding: EdgeInsets.all(24),
        child: Center(child: CircularProgressIndicator(strokeWidth: 2)),
      );
    }
    _mobile = MediaQuery.sizeOf(context).width <= 760;
    if (_mobile) return _mobileLayout(list);
    final current =
        list.where((a) => a.id == selectedId).firstOrNull ?? list.firstOrNull;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        _tabs(list, current),
        if (adding)
          AddAccountPanel(
            key: const ValueKey('add-account-inline'),
            embedded: true,
            onAdded: _added,
          )
        else if (current != null)
          AccountCard(
            key: ValueKey('account-${current.id}'),
            account: current,
            admin: admin,
            setup: setup,
            onChanged: changed,
          ),
      ],
    );
  }

  /// Compte créé dans l'onglet d'ajout : on le sélectionne et on relit la liste.
  Future<void> _added(String id) async {
    await load();
    if (mounted) {
      setState(() {
        adding = false;
        selectedId = id;
        openId = id;
      });
    }
  }

  /// Mobile : la liste des comptes, ou la page d'un compte (ou d'ajout) quand on en ouvre une.
  Widget _mobileLayout(List<AccountDetail> list) {
    final open = list.where((a) => a.id == openId).firstOrNull;
    if (!adding && open == null) return _mobileList(list);
    return _mobileDetail(open);
  }

  /// Une ligne par compte, puis l'ajout d'un compte.
  Widget _mobileList(List<AccountDetail> list) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      for (final a in list)
        AccountListRow(
          key: ValueKey('account-head-${a.id}'),
          account: a,
          stateText: stateLabel(a).$1,
          bad: stateLabel(a).$2,
          onTap: () => setState(() => openId = a.id),
        ),
      AddAccountRow(
        key: const ValueKey('settings-add-account'),
        onTap: () => setState(() => adding = true),
      ),
    ],
  );

  /// En-tête avec retour, puis la page du compte ouvert ou le formulaire d'ajout.
  Widget _mobileDetail(AccountDetail? open) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      _mobileHeader(open),
      if (adding)
        AddAccountPanel(
          key: const ValueKey('add-account-inline'),
          embedded: true,
          onAdded: _added,
        )
      else if (open != null)
        AccountCard(
          key: ValueKey('account-${open.id}'),
          account: open,
          admin: admin,
          setup: setup,
          onChanged: changed,
        ),
    ],
  );

  /// Flèche de retour vers la liste et nom du compte affiché.
  Widget _mobileHeader(AccountDetail? open) {
    final p = context.pal;
    final title = adding
        ? t('Ajouter un compte')
        : (open?.displayName.isEmpty ?? true)
        ? open?.address ?? ''
        : open!.displayName;
    return Padding(
      padding: const EdgeInsets.fromLTRB(8, 8, 8, 4),
      child: Row(
        children: [
          IBtn('arrowl', tip: t('Retour'), onTap: _handleBack),
          const SizedBox(width: 8),
          Expanded(
            child: Text(
              title,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: Fonts.sans(size: 16, w: FontWeight.w500, color: p.text),
            ),
          ),
        ],
      ),
    );
  }

  /// Un onglet par compte, sur une seule ligne qui défile vers la droite ; l'ajout d'un compte reste épinglé au bout.
  Widget _tabs(List<AccountDetail> list, AccountDetail? current) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 18),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.end,
        children: [
          Flexible(
            child: SingleChildScrollView(
              scrollDirection: Axis.horizontal,
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.end,
                children: [
                  for (final a in list)
                    KeyedSubtree(
                      key: ValueKey('account-head-${a.id}'),
                      child: AccountTab(
                        account: a,
                        stateText: stateLabel(a).$1,
                        bad: stateLabel(a).$2,
                        selected: !adding && a.id == current?.id,
                        onTap: () => setState(() {
                          adding = false;
                          selectedId = a.id;
                        }),
                      ),
                    ),
                ],
              ),
            ),
          ),
          AddAccountTab(
            key: const ValueKey('settings-add-account'),
            selected: adding,
            onTap: () => setState(() => adding = true),
          ),
        ],
      ),
    );
  }
}

/// Détails du compte sélectionné : nom, synchro, serveurs, identités, libellés, suppression.
class AccountCard extends StatefulWidget {
  final AccountDetail account;
  final AccountAdmin admin;
  final AccountSetup setup;
  final Future<void> Function() onChanged;
  const AccountCard({
    super.key,
    required this.account,
    required this.admin,
    required this.setup,
    required this.onChanged,
  });

  @override
  State<AccountCard> createState() => _AccountCardState();
}

class _AccountCardState extends State<AccountCard> {
  late final name = TextEditingController(text: widget.account.displayName);
  final password = TextEditingController();
  late final imapHost = TextEditingController(text: widget.account.imap?.host);
  late final imapPort = TextEditingController(
    text: '${widget.account.imap?.port ?? 993}',
  );
  late final smtpHost = TextEditingController(text: widget.account.smtp?.host);
  late final smtpPort = TextEditingController(
    text: '${widget.account.smtp?.port ?? 587}',
  );
  late String imapSec = widget.account.imap?.security ?? 'tls';
  late String smtpSec = widget.account.smtp?.security ?? 'start_tls';
  bool busy = false, confirmDelete = false;
  String? error, info;

  AccountDetail get a => widget.account;

  @override
  void dispose() {
    for (final c in [name, password, imapHost, imapPort, smtpHost, smtpPort]) {
      c.dispose();
    }
    super.dispose();
  }

  Future<void> _run(Future<void> Function() job, {String? done}) async {
    setState(() {
      busy = true;
      error = info = null;
    });
    try {
      await job();
      await widget.onChanged();
      if (mounted && done != null) setState(() => info = done);
    } catch (e) {
      if (mounted) setState(() => error = _message(e));
    } finally {
      if (mounted) setState(() => busy = false);
    }
  }

  Future<void> _saveName() => _run(() {
    final v = name.text.trim();
    if (v.isEmpty) {
      throw EngineException.local(
        'invalid_input.parameter',
        t('Le nom ne peut pas être vide'),
      );
    }
    return widget.admin.rename(a.id, v);
  }, done: t('Nom enregistré.'));

  Future<void> _saveServers() => _run(() {
    final ip = int.tryParse(imapPort.text.trim());
    final sp = int.tryParse(smtpPort.text.trim());
    if (imapHost.text.trim().isEmpty || smtpHost.text.trim().isEmpty) {
      throw EngineException.local(
        'invalid_input.parameter',
        t('Hôte du serveur manquant'),
      );
    }
    if (ip == null ||
        sp == null ||
        ip < 1 ||
        sp < 1 ||
        ip > 65535 ||
        sp > 65535) {
      throw EngineException.local(
        'invalid_input.parameter',
        t('Port invalide (1 à 65535)'),
      );
    }
    return widget.admin.setServers(
      a.id,
      Endpoint(imapHost.text.trim(), ip, imapSec),
      Endpoint(smtpHost.text.trim(), sp, smtpSec),
    );
  }, done: t('Serveurs enregistrés après vérification de la connexion.'));

  Future<void> _reauth() => _run(() async {
    final started = await widget.admin.reauthenticate(a.id);
    final opened = await openLink(started.url);
    if (!opened && mounted) {
      setState(
        () => info = t('Ouvre cette adresse dans ton navigateur : {0}', [
          started.url,
        ]),
      );
    }
    await widget.admin.completeOAuth(started.flow);
  }, done: t('Compte reconnecté.'));

  Future<void> _storePassword() => _run(() async {
    final pw = password.text;
    if (pw.isEmpty) {
      throw EngineException.local(
        'invalid_input.parameter',
        t('Saisis le mot de passe.'),
      );
    }
    await widget.admin.storePassword(a.id, pw);
    password.clear();
  }, done: t('Mot de passe enregistré.'));

  @override
  Widget build(BuildContext context) => _details(context.pal);

  Widget _row(Pal p, String title, Widget child) => Padding(
    padding: const EdgeInsets.only(bottom: 18),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Mono(title, size: 10.5, ls: .12, color: p.muted, height: null),
        const SizedBox(height: 8),
        child,
      ],
    ),
  );

  Widget _details(Pal p) {
    if (MediaQuery.sizeOf(context).width <= 760) return _detailsMobile(p);
    return Padding(
      padding: const EdgeInsets.fromLTRB(20, 4, 20, 8),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          if (a.needsAuth) _reauthBlock(p),
          _row(p, t('NOM DU COMPTE'), _nameEditor()),
          _row(p, t('SYNCHRONISER LES … DERNIERS JOURS'), _windowPicker()),
          _row(
            p,
            t('DOSSIERS SYNCHRONISÉS'),
            SyncedMailboxes(account: a, admin: widget.admin),
          ),
          _row(p, t('COMPTE'), _accountActions()),
          _row(
            p,
            t('ACTIONS EN ATTENTE'),
            PendingActions(account: a, admin: widget.admin),
          ),
          if (a.imap != null) _serversBlock(p),
          IdentitiesBox(account: a, admin: widget.admin),
          _dangerBlock(p),
          ..._feedback(p),
        ],
      ),
    );
  }

  /// Page d'un compte sur mobile : sections repliables, « Général » ouverte.
  Widget _detailsMobile(Pal p) => Padding(
    padding: const EdgeInsets.fromLTRB(20, 4, 20, 8),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        if (a.needsAuth) _reauthBlock(p),
        CollapsibleSection(
          title: t('GÉNÉRAL'),
          initiallyOpen: true,
          child: _generalBody(p),
        ),
        CollapsibleSection(title: t('SYNCHRONISATION'), child: _syncBody(p)),
        if (a.imap != null)
          CollapsibleSection(title: t('SERVEURS'), child: _serversBody(p)),
        CollapsibleSection(
          title: t('IDENTITÉS D’ENVOI'),
          child: IdentitiesBox(
            account: a,
            admin: widget.admin,
            showTitle: false,
          ),
        ),
        CollapsibleSection(title: t('SUPPRESSION'), child: _dangerBody(p)),
        ..._feedback(p),
      ],
    ),
  );

  /// Contenu de « Général » : nom, synchronisation immédiate, actions en attente.
  Widget _generalBody(Pal p) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      _row(p, t('NOM DU COMPTE'), _nameEditor()),
      _row(p, t('COMPTE'), _accountActions()),
      _row(
        p,
        t('ACTIONS EN ATTENTE'),
        PendingActions(account: a, admin: widget.admin),
      ),
    ],
  );

  /// Contenu de « Synchronisation » : fenêtre de jours et dossiers suivis.
  Widget _syncBody(Pal p) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      _row(p, t('SYNCHRONISER LES … DERNIERS JOURS'), _windowPicker()),
      _row(
        p,
        t('DOSSIERS SYNCHRONISÉS'),
        SyncedMailboxes(account: a, admin: widget.admin),
      ),
    ],
  );

  /// Champ du nom du compte et son bouton d'enregistrement.
  Widget _nameEditor() => Wrap(
    spacing: 12,
    runSpacing: 10,
    crossAxisAlignment: WrapCrossAlignment.center,
    children: [
      SizedBox(
        width: 300,
        child: FormField2(
          fieldKey: ValueKey('name-${a.id}'),
          label: '',
          controller: name,
        ),
      ),
      Btn(
        key: ValueKey('save-name-${a.id}'),
        label: t('Enregistrer'),
        onTap: busy ? null : _saveName,
      ),
    ],
  );

  /// Choix de la fenêtre de synchronisation en jours.
  Widget _windowPicker() => Segments<int>(
    keyPrefix: 'window-${a.id}',
    options: const [(30, '30'), (90, '90'), (180, '180'), (365, '365')],
    value: a.windowDays,
    onChanged: busy
        ? (_) {}
        : (d) => _run(
            () => widget.admin.setWindowDays(a.id, d),
            done: t('Fenêtre de synchronisation modifiée.'),
          ),
  );

  /// Boutons de synchronisation et de pause, avec la ligne d'état.
  Widget _accountActions() => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      Wrap(
        spacing: 10,
        runSpacing: 10,
        children: [
          Btn(
            key: ValueKey('sync-${a.id}'),
            icon: 'refresh',
            label: t('Synchroniser maintenant'),
            onTap: busy || a.paused
                ? null
                : () => _run(
                    () => widget.setup.startSync(a.id),
                    done: t('Synchronisation lancée.'),
                  ),
          ),
          Btn(
            key: ValueKey('pause-${a.id}'),
            label: a.paused ? t('Reprendre') : t('Mettre en pause'),
            onTap: busy
                ? null
                : () => _run(
                    () => a.paused
                        ? widget.admin.resume(a.id)
                        : widget.admin.pause(a.id),
                  ),
          ),
        ],
      ),
      SyncStatusLine(account: a, admin: widget.admin),
    ],
  );

  /// Indications d'activité, d'erreur et de succès en bas de la page du compte.
  List<Widget> _feedback(Pal p) => [
    if (busy)
      Padding(
        padding: const EdgeInsets.only(top: 10),
        child: Text(
          t('En cours…'),
          style: Fonts.sans(size: 14, color: p.muted),
        ),
      ),
    if (error != null)
      Padding(
        padding: const EdgeInsets.only(top: 10),
        child: Text(
          error!,
          key: ValueKey('error-${a.id}'),
          style: Fonts.sans(size: 14, color: p.accent),
        ),
      ),
    if (info != null)
      Padding(
        padding: const EdgeInsets.only(top: 10),
        child: SelectableText(
          info!,
          key: ValueKey('info-${a.id}'),
          style: Fonts.sans(size: 14, color: p.muted),
        ),
      ),
    const SizedBox(height: 10),
  ];

  Widget _reauthBlock(Pal p) => Container(
    margin: const EdgeInsets.only(bottom: 18),
    padding: const EdgeInsets.all(14),
    decoration: BoxDecoration(
      color: p.accentSoft,
      border: Border.all(color: p.accent),
    ),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          t('Ce compte doit être reconnecté.'),
          style: Fonts.sans(size: 14.5, w: FontWeight.w500, color: p.text),
        ),
        const SizedBox(height: 10),
        if (a.auth == 'oauth2')
          Btn(
            key: ValueKey('reauth-${a.id}'),
            icon: 'send',
            label: t('Se reconnecter'),
            go: true,
            onTap: busy ? null : _reauth,
          )
        else
          Row(
            children: [
              SizedBox(
                width: 300,
                child: FormField2(
                  fieldKey: ValueKey('newpw-${a.id}'),
                  label: t('NOUVEAU MOT DE PASSE'),
                  controller: password,
                  secret: true,
                ),
              ),
              const SizedBox(width: 12),
              Btn(
                key: ValueKey('storepw-${a.id}'),
                label: t('Enregistrer'),
                go: true,
                onTap: busy ? null : _storePassword,
              ),
            ],
          ),
      ],
    ),
  );

  Widget _serversBlock(Pal p) => _row(p, t('SERVEURS'), _serversBody(p));

  /// Champs IMAP et SMTP du compte avec le bouton d'enregistrement.
  Widget _serversBody(Pal p) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      ServerFields(
        title: t('RÉCEPTION · IMAP'),
        keyPrefix: 'imap-${a.id}',
        host: imapHost,
        port: imapPort,
        security: imapSec,
        onSecurity: (v) => setState(() => imapSec = v),
        onEdited: () {},
      ),
      const SizedBox(height: 16),
      ServerFields(
        title: t('ENVOI · SMTP'),
        keyPrefix: 'smtp-${a.id}',
        host: smtpHost,
        port: smtpPort,
        security: smtpSec,
        onSecurity: (v) => setState(() => smtpSec = v),
        onEdited: () {},
      ),
      const SizedBox(height: 14),
      Btn(
        key: ValueKey('save-servers-${a.id}'),
        label: t('Enregistrer les serveurs'),
        onTap: busy ? null : _saveServers,
      ),
    ],
  );

  Widget _dangerBlock(Pal p) => _row(p, t('SUPPRESSION'), _dangerBody(p));

  /// Bouton de suppression du compte, puis sa confirmation.
  Widget _dangerBody(Pal p) => confirmDelete
      ? Container(
          padding: const EdgeInsets.all(14),
          decoration: BoxDecoration(border: Border.all(color: p.accent)),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                a.pendingActions > 0
                    ? t('{0} action(s) en attente seront perdues. ', [
                            '${a.pendingActions}',
                          ]) +
                          t(
                            'Messages locaux, brouillons et identifiants de ce compte seront effacés de MIRA ; le courrier reste sur le serveur.',
                          )
                    : t(
                        'Messages locaux, brouillons et identifiants de ce compte seront effacés de MIRA ; le courrier reste sur le serveur.',
                      ),
                style: Fonts.sans(size: 14, color: p.text),
              ),
              const SizedBox(height: 12),
              Wrap(
                spacing: 10,
                children: [
                  Btn(
                    key: ValueKey('confirm-delete-${a.id}'),
                    icon: 'trash',
                    label: t('Supprimer définitivement'),
                    go: true,
                    onTap: busy
                        ? null
                        : () => _run(() => widget.admin.delete(a.id)),
                  ),
                  Btn(
                    label: t('Annuler'),
                    onTap: () => setState(() => confirmDelete = false),
                  ),
                ],
              ),
            ],
          ),
        )
      : Btn(
          key: ValueKey('delete-${a.id}'),
          icon: 'trash',
          label: t('Supprimer ce compte'),
          onTap: () => setState(() => confirmDelete = true),
        );
}
