import 'dart:async';

import 'package:flutter/material.dart';

import '../engine/account_admin.dart';

import 'package:provider/provider.dart';

import '../engine/engine_client.dart';
import '../state/alias_hints.dart';
import '../state/store.dart';
import '../i18n/engine_errors.dart';
import '../i18n/i18n.dart';
import '../theme/app_theme.dart';
import 'account_form_widgets.dart';
import 'widgets.dart';

final _addr = RegExp(r'^[^\s@<>,;]+@[^\s@<>,;]+\.[^\s@<>,;]+$');

/// Identités d'envoi d'un compte : liste, modification du nom et de la signature, identité par défaut, ajout.
class IdentitiesBox extends StatefulWidget {
  final AccountDetail account;
  final AccountAdmin admin;
  const IdentitiesBox({super.key, required this.account, required this.admin});

  @override
  State<IdentitiesBox> createState() => _IdentitiesBoxState();
}

class _IdentitiesBoxState extends State<IdentitiesBox> {
  List<IdentityDetail>? items;
  String? error;
  final newAddress = TextEditingController();
  final newName = TextEditingController();
  final newSignature = TextEditingController();
  bool newDefault = false, busy = false;

  @override
  void initState() {
    super.initState();
    unawaited(load());
  }

  @override
  void dispose() {
    newAddress.dispose();
    newName.dispose();
    newSignature.dispose();
    super.dispose();
  }

  Future<void> load() async {
    try {
      final list = await widget.admin.identities(widget.account.id);
      if (mounted) setState(() => items = list);
    } catch (e) {
      if (mounted) setState(() => error = _msg(e));
    }
  }

  String _msg(Object e) => engineErrorText(e);

  Future<void> _run(Future<void> Function() job, {String? done}) async {
    setState(() {
      busy = true;
      error = null;
    });
    try {
      await job();
      await load();
      if (mounted) await context.read<MailStore>().identitiesChanged();
      if (mounted && done != null) context.read<MailStore>().toast(done);
    } catch (e) {
      if (mounted) setState(() => error = _msg(e));
    } finally {
      if (mounted) setState(() => busy = false);
    }
  }

  Future<void> _add() => _run(() async {
    final address = newAddress.text.trim();
    if (!_addr.hasMatch(address)) {
      throw EngineException.local(
        'invalid_input.address',
        t('Adresse e-mail invalide.'),
      );
    }
    final name = newName.text.trim();
    await widget.admin.addIdentity(
      accountId: widget.account.id,
      address: address,
      displayName: name.isEmpty ? address : name,
      signature: newSignature.text.isEmpty ? null : newSignature.text,
      isDefault: newDefault,
    );
    newAddress.clear();
    newName.clear();
    newSignature.clear();
    newDefault = false;
  }, done: t('Identité ajoutée'));

  /// Adresses vues dans les messages déjà synchronisés, proposées en un clic.
  Widget _hints(Pal p, List<IdentityDetail> list) {
    final known = {
      widget.account.address.toLowerCase(),
      for (final i in list) i.address.toLowerCase(),
    };
    final hints = aliasHints(
      context.read<MailStore>().mailsOf(widget.account.id),
      known,
    );
    if (hints.isEmpty) return const SizedBox.shrink();
    return Container(
      margin: const EdgeInsets.only(top: 4),
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        color: p.accentSoft,
        border: Border.all(color: p.line),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            t('Adresses détectées dans tes messages'),
            style: Fonts.sans(size: 14, w: FontWeight.w500, color: p.text),
          ),
          Text(
            t(
              'Le courrier ne liste pas tes alias : MIRA les devine des messages envoyés et reçus. Un alias doit aussi être déclaré chez ton fournisseur pour que l’envoi sous cette adresse soit accepté.',
            ),
            style: Fonts.sans(size: 13, color: p.dim),
          ),
          const SizedBox(height: 10),
          for (final h in hints) _hintRow(p, h),
        ],
      ),
    );
  }

  /// Une adresse détectée avec ses compteurs et son bouton d'ajout ; empilée sur mobile.
  Widget _hintRow(Pal p, AliasHint h) {
    final address = Text(
      h.address,
      style: Fonts.sans(size: 14.5, color: p.text),
    );
    final stats = Mono(
      [
        if (h.sentAs > 0) t('expéditeur ×{0}', ['${h.sentAs}']),
        if (h.receivedAt > 0) t('destinataire ×{0}', ['${h.receivedAt}']),
      ].join(' · '),
      size: 10.5,
      color: p.muted,
      height: null,
    );
    final add = Btn(
      key: ValueKey('hint-${h.address}'),
      icon: 'plus',
      label: t('Ajouter'),
      onTap: busy ? null : () => _addHint(h),
    );
    final mobile = MediaQuery.sizeOf(context).width <= 760;
    return Padding(
      padding: const EdgeInsets.only(bottom: 6),
      child: mobile
          ? Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                address,
                const SizedBox(height: 4),
                Row(
                  children: [
                    Expanded(child: stats),
                    add,
                  ],
                ),
              ],
            )
          : Row(
              children: [
                Expanded(child: address),
                stats,
                const SizedBox(width: 10),
                add,
              ],
            ),
    );
  }

  /// Ajoute l'adresse détectée comme identité du compte.
  Future<void> _addHint(AliasHint h) => _run(
    () => widget.admin.addIdentity(
      accountId: widget.account.id,
      address: h.address,
      displayName: widget.account.displayName,
    ),
  );

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final list = items;
    return Padding(
      padding: const EdgeInsets.only(bottom: 18),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Mono(
            t('IDENTITÉS D’ENVOI'),
            size: 10.5,
            ls: .12,
            color: p.muted,
            height: null,
          ),
          const SizedBox(height: 8),
          if (list == null)
            const SizedBox(
              height: 24,
              child: Center(child: LinearProgressIndicator()),
            )
          else ...[
            for (final i in list)
              _IdentityRow(
                key: ValueKey('identity-${i.id}'),
                identity: i,
                busy: busy,
                onSave: (name, sig) => _run(
                  () => widget.admin.updateIdentity(
                    i.id,
                    displayName: name,
                    signature: sig,
                  ),
                  done: t('Identité enregistrée'),
                ),
                onDefault: () => _run(
                  () => widget.admin.setDefaultIdentity(i.id),
                  done: t('Identité par défaut modifiée'),
                ),
              ),
            if (list.isEmpty)
              Text(
                t('Aucune identité.'),
                style: Fonts.sans(size: 13.5, color: p.dim),
              ),
          ],
          if (list != null) _hints(p, list),
          const SizedBox(height: 12),
          Container(
            padding: const EdgeInsets.all(14),
            decoration: BoxDecoration(
              color: p.bg,
              border: Border.all(color: p.line),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  t('Ajouter une identité (alias, autre nom)'),
                  style: Fonts.sans(
                    size: 14,
                    w: FontWeight.w500,
                    color: p.text,
                  ),
                ),
                const SizedBox(height: 10),
                SideBySide(
                  leftWidth: 280,
                  left: Column(
                    children: [
                      FormField2(
                        fieldKey: ValueKey(
                          'new-id-address-${widget.account.id}',
                        ),
                        label: t('ADRESSE'),
                        controller: newAddress,
                        hint: 'alias@exemple.com',
                      ),
                      const SizedBox(height: 10),
                      FormField2(
                        fieldKey: ValueKey('new-id-name-${widget.account.id}'),
                        label: t('NOM AFFICHÉ'),
                        controller: newName,
                      ),
                    ],
                  ),
                  right: FormField2(
                    fieldKey: ValueKey('new-id-sig-${widget.account.id}'),
                    label: t('SIGNATURE'),
                    controller: newSignature,
                    maxLines: 5,
                  ),
                ),
                const SizedBox(height: 10),
                Wrap(
                  spacing: 14,
                  crossAxisAlignment: WrapCrossAlignment.center,
                  children: [
                    Segments<bool>(
                      keyPrefix: 'new-id-default-${widget.account.id}',
                      options: [
                        (false, t('Secondaire')),
                        (true, t('Par défaut')),
                      ],
                      value: newDefault,
                      onChanged: (v) => setState(() => newDefault = v),
                    ),
                    Btn(
                      key: ValueKey('add-identity-${widget.account.id}'),
                      icon: 'plus',
                      label: t('Ajouter'),
                      onTap: busy ? null : _add,
                    ),
                  ],
                ),
              ],
            ),
          ),
          if (error != null)
            Padding(
              padding: const EdgeInsets.only(top: 10),
              child: Text(
                error!,
                key: ValueKey('identity-error-${widget.account.id}'),
                style: Fonts.sans(size: 14, color: p.accent),
              ),
            ),
        ],
      ),
    );
  }
}

class _IdentityRow extends StatefulWidget {
  final IdentityDetail identity;
  final bool busy;
  final void Function(String name, String? signature) onSave;
  final VoidCallback onDefault;
  const _IdentityRow({
    super.key,
    required this.identity,
    required this.busy,
    required this.onSave,
    required this.onDefault,
  });

  @override
  State<_IdentityRow> createState() => _IdentityRowState();
}

class _IdentityRowState extends State<_IdentityRow> {
  late final name = TextEditingController(text: widget.identity.displayName);
  late final sig = TextEditingController(text: widget.identity.signature ?? '');

  @override
  void dispose() {
    name.dispose();
    sig.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final i = widget.identity;
    return Container(
      margin: const EdgeInsets.only(bottom: 10),
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        border: Border.all(color: i.isDefault ? p.accent : p.line),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Expanded(
                child: Text(
                  i.address,
                  style: Fonts.sans(
                    size: 15,
                    w: FontWeight.w500,
                    color: p.text,
                  ),
                ),
              ),
              if (i.isDefault)
                Mono(
                  t('PAR DÉFAUT'),
                  size: 10.5,
                  ls: .12,
                  color: p.accent,
                  height: null,
                )
              else
                Btn(
                  key: ValueKey('default-${i.id}'),
                  label: t('Définir par défaut'),
                  onTap: widget.busy ? null : widget.onDefault,
                ),
            ],
          ),
          const SizedBox(height: 10),
          SideBySide(
            leftWidth: 260,
            left: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                FormField2(
                  fieldKey: ValueKey('id-name-${i.id}'),
                  label: t('NOM AFFICHÉ'),
                  controller: name,
                ),
                const SizedBox(height: 10),
                Btn(
                  key: ValueKey('save-identity-${i.id}'),
                  label: t('Enregistrer'),
                  onTap: widget.busy
                      ? null
                      : () => widget.onSave(
                          name.text.trim().isEmpty
                              ? i.displayName
                              : name.text.trim(),
                          sig.text,
                        ),
                ),
              ],
            ),
            right: FormField2(
              fieldKey: ValueKey('id-sig-${i.id}'),
              label: t('SIGNATURE'),
              controller: sig,
              maxLines: 5,
            ),
          ),
        ],
      ),
    );
  }
}
