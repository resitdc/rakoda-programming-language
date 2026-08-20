import 'package:flutter/widgets.dart';

/// Model for open editor tabs.
class EditorTab {
  final String filePath;
  final String fileName;
  bool isModified;
  String content;
  int cursorLine;
  int cursorColumn;
  final UndoHistoryController undoController;

  EditorTab({
    required this.filePath,
    required this.fileName,
    this.isModified = false,
    this.content = '',
    this.cursorLine = 1,
    this.cursorColumn = 1,
  }) : undoController = UndoHistoryController();

  String get title => isModified ? '• $fileName' : fileName;
}
