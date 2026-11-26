//! Lecture du texte d'un fichier à joindre à l'assistant : texte brut ou PDF, avec une taille maximale.

import 'dart:convert';
import 'dart:typed_data';

import 'package:pdfrx/pdfrx.dart';

import 'chat_attached_file.dart';

/// Taille maximale d'un fichier joint, en octets.
const chatFileMaxBytes = 10 * 1024 * 1024;

/// Texte d'un contenu en octets : `null` quand ce n'est pas du texte (octet nul ou UTF-8 invalide).
String? plainTextOf(List<int> bytes) {
  if (bytes.contains(0)) return null;
  try {
    return utf8.decode(bytes);
  } on FormatException {
    return null;
  }
}

/// Texte de toutes les pages d'un PDF, une page après l'autre.
Future<String> pdfTextOf(Uint8List bytes) async {
  final doc = await PdfDocument.openData(bytes);
  try {
    final out = StringBuffer();
    for (final page in doc.pages) {
      final text = await page.loadText();
      if (text != null) out.writeln(text.fullText);
    }
    return out.toString();
  } finally {
    await doc.dispose();
  }
}

/// Vrai quand le nom ou les premiers octets indiquent un PDF.
bool looksLikePdf(String name, Uint8List bytes) =>
    name.toLowerCase().endsWith('.pdf') ||
    (bytes.length > 4 &&
        utf8.decode(bytes.sublist(0, 5), allowMalformed: true) == '%PDF-');

/// Lit un fichier joint ; lève [ChatFileException] s'il est trop gros, illisible ou vide.
Future<ChatAttachedFile> readChatFile(
  String id,
  String name,
  Uint8List bytes,
) async {
  if (bytes.length > chatFileMaxBytes) {
    throw const ChatFileException(ChatFileProblem.tooLarge);
  }
  String? text;
  try {
    text = looksLikePdf(name, bytes)
        ? await pdfTextOf(bytes)
        : plainTextOf(bytes);
  } on Object {
    text = null;
  }
  if (text == null) throw const ChatFileException(ChatFileProblem.unreadable);
  if (text.trim().isEmpty) throw const ChatFileException(ChatFileProblem.empty);
  return ChatAttachedFile(id: id, name: name, text: text);
}
