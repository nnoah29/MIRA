//! Choix de fichiers à joindre à l'assistant : sélecteur du système, lecture du texte, message d'erreur par fichier.

import 'package:file_picker/file_picker.dart';

import '../i18n/i18n.dart';
import '../state/chat_attached_file.dart';
import '../state/chat_file_reader.dart';
import '../state/store.dart';

/// Message dit à l'utilisateur quand un fichier ne peut pas être joint.
String chatFileProblemText(ChatFileProblem problem, String name) =>
    switch (problem) {
      ChatFileProblem.tooLarge => t(
        'Fichier trop volumineux (10 Mo au plus) : {0}',
        [name],
      ),
      ChatFileProblem.unreadable => t(
        'Ce fichier n’est ni du texte ni un PDF lisible : {0}',
        [name],
      ),
      ChatFileProblem.empty => t('Aucun texte trouvé dans ce fichier : {0}', [
        name,
      ]),
    };

/// Ouvre le sélecteur et joint chaque fichier lisible ; les autres donnent un message d'erreur.
Future<void> pickChatFiles(MailStore store) async {
  final picked = await FilePicker.pickFiles();
  for (final f in picked) {
    try {
      final id = '${DateTime.now().microsecondsSinceEpoch}-${f.name}';
      store.attachChatFile(
        await readChatFile(id, f.name, await f.readAsBytes()),
      );
    } on ChatFileException catch (e) {
      store.toast(chatFileProblemText(e.problem, f.name), err: true);
    }
  }
}
