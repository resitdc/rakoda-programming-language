import '../../models/database_connection.dart';
import 'sqlite_service.dart';
import 'postgres_service.dart';
import 'mysql_service.dart';

class QueryResult {
  final List<String> columns;
  final List<Map<String, dynamic>> rows;
  final int affectedRows;
  final String? error;

  QueryResult({
    required this.columns,
    required this.rows,
    this.affectedRows = 0,
    this.error,
  });
}

abstract class DatabaseService {
  Future<void> connect();
  Future<void> disconnect();

  Future<List<String>> getDatabases();
  Future<List<String>> getSchemas(String database);
  Future<List<String>> getTables(String database, String schema);
  Future<List<Map<String, dynamic>>> getTableStructure(String database, String schema, String table);
  
  Future<QueryResult> executeQuery(String query);
  
  /// Factory to get the correct service implementation
  static DatabaseService fromConnection(DatabaseConnection connection) {
    switch (connection.engine) {
      case DatabaseEngine.sqlite:
        return SqliteService(connection);
      case DatabaseEngine.postgres:
        return PostgresService(connection);
      case DatabaseEngine.mysql:
        return MysqlService(connection);
    }
  }
}
