import 'dart:convert';
import 'dart:io';
import 'package:path/path.dart' as p;
import '../../models/database_connection.dart';

class ConnectionService {
  static const String _fileName = 'connections.json';

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

  static Future<List<DatabaseConnection>> getConnections(String projectPath) async {
    final file = File(_getFilePath(projectPath));
    if (!file.existsSync()) return [];
    try {
      final content = await file.readAsString();
      final List<dynamic> list = jsonDecode(content);
      return list.map((e) => DatabaseConnection.fromJson(e as Map<String, dynamic>)).toList();
    } catch (_) {
      return [];
    }
  }

  static Future<void> saveConnection(String projectPath, DatabaseConnection connection) async {
    final connections = await getConnections(projectPath);
    final index = connections.indexWhere((c) => c.id == connection.id);

    if (index >= 0) {
      connections[index] = connection;
    } else {
      connections.add(connection);
    }

    await _saveAll(projectPath, connections);
  }

  static Future<void> deleteConnection(String projectPath, String id) async {
    final connections = await getConnections(projectPath);
    connections.removeWhere((c) => c.id == id);
    await _saveAll(projectPath, connections);
  }

  static Future<void> _saveAll(String projectPath, List<DatabaseConnection> connections) async {
    _ensureDir(projectPath);
    final file = File(_getFilePath(projectPath));
    final jsonList = connections.map((c) => c.toJson()).toList();
    await file.writeAsString(jsonEncode(jsonList), flush: true);
  }
}
