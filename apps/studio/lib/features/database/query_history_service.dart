import 'dart:convert';
import 'dart:io';
import 'package:path/path.dart' as p;

class QueryHistoryItem {
  final String query;
  final DateTime executedAt;
  final String connectionName;
  final String? database;

  QueryHistoryItem({
    required this.query,
    required this.executedAt,
    required this.connectionName,
    this.database,
  });

  Map<String, dynamic> toJson() => {
        'query': query,
        'executedAt': executedAt.toIso8601String(),
        'connectionName': connectionName,
        'database': database,
      };

  factory QueryHistoryItem.fromJson(Map<String, dynamic> json) => QueryHistoryItem(
        query: json['query'],
        executedAt: DateTime.parse(json['executedAt']),
        connectionName: json['connectionName'],
        database: json['database'],
      );
}

class QueryHistoryService {
  static const String _fileName = 'query_history.json';

  static String _getFilePath(String projectPath) {
    return p.join(projectPath, '.rpl_studio', _fileName);
  }

  static void _ensureDir(String projectPath) {
    final dir = Directory(p.join(projectPath, '.rpl_studio'));
    if (!dir.existsSync()) {
      dir.createSync();
      if (Platform.isWindows) {
        try {
          Process.runSync('attrib', ['+h', dir.path]);
        } catch (_) {}
      }
    }
  }

  static Future<List<QueryHistoryItem>> getHistory(String projectPath) async {
    final file = File(_getFilePath(projectPath));
    if (!file.existsSync()) return [];
    try {
      final content = await file.readAsString();
      final List<dynamic> list = jsonDecode(content);
      return list.map((e) => QueryHistoryItem.fromJson(e as Map<String, dynamic>)).toList();
    } catch (_) {
      return [];
    }
  }

  static Future<void> addHistory(String projectPath, QueryHistoryItem item) async {
    final history = await getHistory(projectPath);
    history.insert(0, item);
    _ensureDir(projectPath);
    final file = File(_getFilePath(projectPath));
    await file.writeAsString(
        jsonEncode(history.map((e) => e.toJson()).toList()),
        flush: true);
  }

  static Future<void> clearHistory(String projectPath) async {
    final file = File(_getFilePath(projectPath));
    if (file.existsSync()) {
      await file.delete();
    }
  }
}
