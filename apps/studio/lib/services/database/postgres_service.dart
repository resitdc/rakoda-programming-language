import 'package:postgres/postgres.dart';
import '../../models/database_connection.dart';
import 'database_service.dart';

class PostgresService extends DatabaseService {
  final DatabaseConnection connection;
  Connection? _db;

  PostgresService(this.connection);

  @override
  Future<void> connect() async {
    _db = await Connection.open(
      Endpoint(
        host: connection.host ?? 'localhost',
        port: connection.port ?? 5432,
        database: connection.database ?? 'postgres',
        username: connection.username,
        password: connection.password,
      ),
      settings: const ConnectionSettings(sslMode: SslMode.disable),
    );
  }

  @override
  Future<void> disconnect() async {
    await _db?.close();
    _db = null;
  }

  @override
  Future<List<String>> getDatabases() async {
    final result = await executeQuery("SELECT datname FROM pg_database WHERE datistemplate = false;");
    return result.rows.map((row) => row['datname'] as String).toList();
  }

  @override
  Future<List<String>> getSchemas(String database) async {
    final result = await executeQuery("SELECT schema_name FROM information_schema.schemata WHERE schema_name NOT IN ('information_schema', 'pg_catalog', 'pg_toast');");
    return result.rows.map((row) => row['schema_name'] as String).toList();
  }

  @override
  Future<List<String>> getTables(String database, String schema) async {
    final result = await executeQuery("SELECT table_name FROM information_schema.tables WHERE table_schema = '$schema' AND table_type = 'BASE TABLE';");
    return result.rows.map((row) => row['table_name'] as String).toList();
  }

  @override
  Future<List<Map<String, dynamic>>> getTableStructure(String database, String schema, String table) async {
    final result = await executeQuery("SELECT column_name, data_type, is_nullable, column_default FROM information_schema.columns WHERE table_schema = '$schema' AND table_name = '$table';");
    return result.rows;
  }

  @override
  Future<QueryResult> executeQuery(String query) async {
    if (_db == null) throw Exception('Not connected');

    try {
      final result = await _db!.execute(query);
      
      final columns = result.schema.columns.map((c) => c.columnName ?? '').toList();
      final rows = <Map<String, dynamic>>[];
      
      for (final row in result) {
        final map = <String, dynamic>{};
        for (int i = 0; i < columns.length; i++) {
          map[columns[i]] = row[i];
        }
        rows.add(map);
      }
      
      return QueryResult(
        columns: columns,
        rows: rows,
        affectedRows: result.affectedRows,
      );
    } catch (e) {
      return QueryResult(columns: [], rows: [], error: e.toString());
    }
  }
}
