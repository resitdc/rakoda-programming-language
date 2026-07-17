enum DatabaseEngine {
  sqlite,
  mysql,
  postgres,
}

class DatabaseConnection {
  final String id;
  final String name;
  final DatabaseEngine engine;
  final String? host;
  final int? port;
  final String? username;
  final String? password;
  final String? database;
  final String? sqlitePath;

  DatabaseConnection({
    required this.id,
    required this.name,
    required this.engine,
    this.host,
    this.port,
    this.username,
    this.password,
    this.database,
    this.sqlitePath,
  });

  factory DatabaseConnection.fromJson(Map<String, dynamic> json) {
    return DatabaseConnection(
      id: json['id'] as String,
      name: json['name'] as String,
      engine: DatabaseEngine.values.firstWhere(
        (e) => e.name == json['engine'],
        orElse: () => DatabaseEngine.sqlite,
      ),
      host: json['host'] as String?,
      port: json['port'] as int?,
      username: json['username'] as String?,
      password: json['password'] as String?,
      database: json['database'] as String?,
      sqlitePath: json['sqlitePath'] as String?,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'name': name,
      'engine': engine.name,
      'host': host,
      'port': port,
      'username': username,
      'password': password,
      'database': database,
      'sqlitePath': sqlitePath,
    };
  }

  DatabaseConnection copyWith({
    String? id,
    String? name,
    DatabaseEngine? engine,
    String? host,
    int? port,
    String? username,
    String? password,
    String? database,
    String? sqlitePath,
  }) {
    return DatabaseConnection(
      id: id ?? this.id,
      name: name ?? this.name,
      engine: engine ?? this.engine,
      host: host ?? this.host,
      port: port ?? this.port,
      username: username ?? this.username,
      password: password ?? this.password,
      database: database ?? this.database,
      sqlitePath: sqlitePath ?? this.sqlitePath,
    );
  }
}
