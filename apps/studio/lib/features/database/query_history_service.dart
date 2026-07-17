import 'dart:convert';
import 'package:shared_preferences/shared_preferences.dart';

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
  static const String _key = 'rpl_studio_query_history';

  static Future<List<QueryHistoryItem>> getHistory() async {
    final prefs = await SharedPreferences.getInstance();
    final String? data = prefs.getString(_key);
    if (data == null) return [];
    try {
      final List<dynamic> list = jsonDecode(data);
      return list.map((e) => QueryHistoryItem.fromJson(e)).toList();
    } catch (_) {
      return [];
    }
  }

  static Future<void> addHistory(QueryHistoryItem item) async {
    final history = await getHistory();
    history.insert(0, item);
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(
        _key, jsonEncode(history.map((e) => e.toJson()).toList()));
  }

  static Future<void> clearHistory() async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.remove(_key);
  }
}
