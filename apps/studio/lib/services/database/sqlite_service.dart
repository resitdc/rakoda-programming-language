import 'package:sqflite_common_ffi/sqflite_ffi.dart';
import '../../models/database_connection.dart';
import 'database_service.dart';

bool _ffiInitialized = false;

class SqliteService extends DatabaseService {
  final DatabaseConnection connection;
  Database? _db;

  SqliteService(this.connection);

  @override
  Future<void> connect() async {
    if (!_ffiInitialized) {
      sqfliteFfiInit();
      _ffiInitialized = true;
    }
    final databaseFactory = databaseFactoryFfi;
    final path = connection.sqlitePath;
    if (path == null || path.isEmpty) {
      throw Exception('SQLite path is required');
    }
    try {
      _db = await databaseFactory.openDatabase(
        path,
        options: OpenDatabaseOptions(singleInstance: false),
      );
    } catch (e) {
      throw Exception('Failed to open database at path: $path\n$e');
    }
  }

  @override
  Future<void> disconnect() async {
    await _db?.close();
    _db = null;
  }

  @override
  Future<List<String>> getDatabases() async {
    return ['main'];
  }

  @override
  Future<List<String>> getSchemas(String database) async {
    return ['default'];
  }

  @override
  Future<List<String>> getTables(String database, String schema) async {
    final result = await executeQuery("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'");
    return result.rows.map((row) => row['name'] as String).toList();
  }

  @override
  Future<List<Map<String, dynamic>>> getTableStructure(String database, String schema, String table) async {
    final result = await executeQuery("PRAGMA table_info('$table')");
    return result.rows;
  }

  @override
  Future<QueryResult> executeQuery(String query) async {
    if (_db == null) throw Exception('Not connected');
    
    try {
      final isSelect = query.trimLeft().toUpperCase().startsWith('SELECT') || query.trimLeft().toUpperCase().startsWith('PRAGMA');
      if (isSelect) {
        final List<Map<String, Object?>> maps = await _db!.rawQuery(query);
        if (maps.isEmpty) {
          return QueryResult(columns: [], rows: []);
        }
        return QueryResult(
          columns: maps.first.keys.toList(),
          rows: maps,
        );
      } else {
        final count = await _db!.rawUpdate(query);
        return QueryResult(columns: [], rows: [], affectedRows: count);
      }
    } catch (e) {
      final pathInfo = _db == null ? ' (Path: ${connection.sqlitePath})' : '';
      return QueryResult(columns: [], rows: [], error: '${e.toString()}$pathInfo');
    }
  }
}
