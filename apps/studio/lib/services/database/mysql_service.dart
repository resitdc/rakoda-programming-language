import 'package:mysql_client/mysql_client.dart';
import '../../models/database_connection.dart';
import 'database_service.dart';

class MysqlService extends DatabaseService {
  final DatabaseConnection connection;
  MySQLConnection? _db;

  MysqlService(this.connection);

  @override
  Future<void> connect() async {
    _db = await MySQLConnection.createConnection(
      host: connection.host ?? '127.0.0.1',
      port: connection.port ?? 3306,
      userName: connection.username ?? 'root',
      password: connection.password ?? '',
      databaseName: connection.database, // Optional
    );
    await _db!.connect();
  }

  @override
  Future<void> disconnect() async {
    await _db?.close();
    _db = null;
  }

  @override
  Future<List<String>> getDatabases() async {
    final result = await executeQuery("SHOW DATABASES");
    return result.rows.map((row) => row.values.first as String).toList();
  }

  @override
  Future<List<String>> getSchemas(String database) async {
    return [database];
  }

  @override
  Future<List<String>> getTables(String database, String schema) async {
    final result = await executeQuery("SHOW TABLES FROM `$database`");
    return result.rows.map((row) => row.values.first as String).toList();
  }

  @override
  Future<List<Map<String, dynamic>>> getTableStructure(String database, String schema, String table) async {
    final result = await executeQuery("DESCRIBE `$database`.`$table`");
    return result.rows;
  }

  @override
  Future<QueryResult> executeQuery(String query) async {
    if (_db == null) throw Exception('Not connected');
    
    try {
      final result = await _db!.execute(query);
      
      final columns = result.cols.map((c) => c.name).toList();
      final rows = <Map<String, dynamic>>[];
      
      for (final row in result.rows) {
        final map = <String, dynamic>{};
        for (int i = 0; i < columns.length; i++) {
          map[columns[i]] = row.colAt(i);
        }
        rows.add(map);
      }
      
      return QueryResult(
        columns: columns,
        rows: rows,
        affectedRows: result.affectedRows.toInt(),
      );
    } catch (e) {
      return QueryResult(columns: [], rows: [], error: e.toString());
    }
  }
}
