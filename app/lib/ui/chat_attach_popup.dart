//! Popover de jonction de l'assistant : menu d'options puis liste des mails ; se ferme d'un clic à côté.

import 'package:flutter/material.dart';

import '../i18n/i18n.dart';
import '../state/store.dart';
import 'chat_attach_menu.dart';
import 'chat_file_picker.dart';
import 'chat_mail_picker.dart';

/// Contenu du popover : étape menu, puis étape liste des mails (qui suit les cases cochées).
class ChatAttachPopup extends StatefulWidget {
  final MailStore store;
  final VoidCallback close;
  const ChatAttachPopup({super.key, required this.store, required this.close});

  @override
  State<ChatAttachPopup> createState() => _ChatAttachPopupState();
}

class _ChatAttachPopupState extends State<ChatAttachPopup> {
  bool listing = false;

  void _pickFile() {
    final store = widget.store;
    widget.close();
    pickChatFiles(store);
  }

  @override
  Widget build(BuildContext context) {
    if (!listing) {
      return ChatAttachMenu(
        options: [
          ChatAttachOption(
            id: 'mail',
            icon: 'inbox',
            label: t('Joindre un mail'),
            onTap: () => setState(() => listing = true),
          ),
          ChatAttachOption(
            id: 'file',
            icon: 'file',
            label: t('Joindre un fichier'),
            onTap: _pickFile,
          ),
        ],
      );
    }
    return ListenableBuilder(
      listenable: widget.store,
      builder: (_, _) => ChatMailPicker(store: widget.store),
    );
  }
}
