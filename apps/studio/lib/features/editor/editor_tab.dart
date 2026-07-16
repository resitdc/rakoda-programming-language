import 'package:flutter/widgets.dart';

/// Model for open editor tabs.
class EditorTab {
  final String filePath;
  final String fileName;
  bool isModified;
  String content;
  final UndoHistoryController undoController;

  EditorTab({
    required this.filePath,
    required this.fileName,
    this.isModified = false,
    this.content = '',
  }) : undoController = UndoHistoryController();

  String get title => isModified ? '• $fileName' : fileName;
}
