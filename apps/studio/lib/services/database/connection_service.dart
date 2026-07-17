import 'dart:convert';
import 'package:shared_preferences/shared_preferences.dart';
import '../../models/database_connection.dart';

class ConnectionService {
  static const String _connectionsKey = 'database_connections';

  static Future<List<DatabaseConnection>> getConnections() async {
    final prefs = await SharedPreferences.getInstance();
    final connectionsJson = prefs.getStringList(_connectionsKey) ?? [];
    return connectionsJson.map((jsonStr) {
      final map = jsonDecode(jsonStr) as Map<String, dynamic>;
      return DatabaseConnection.fromJson(map);
    }).toList();
  }

  static Future<void> saveConnection(DatabaseConnection connection) async {
    final connections = await getConnections();
    final index = connections.indexWhere((c) => c.id == connection.id);
    
    if (index >= 0) {
      connections[index] = connection;
    } else {
      connections.add(connection);
    }
    
    await _saveAll(connections);
  }

  static Future<void> deleteConnection(String id) async {
    final connections = await getConnections();
    connections.removeWhere((c) => c.id == id);
    await _saveAll(connections);
  }

  static Future<void> _saveAll(List<DatabaseConnection> connections) async {
    final prefs = await SharedPreferences.getInstance();
    final jsonList = connections.map((c) => jsonEncode(c.toJson())).toList();
    await prefs.setStringList(_connectionsKey, jsonList);
  }
}
