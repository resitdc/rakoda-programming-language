import 'package:flutter/material.dart';
import 'query_history_service.dart';
import 'package:flutter_code_editor/flutter_code_editor.dart';
import 'package:flutter_highlight/themes/vs2015.dart';
import 'package:highlight/languages/sql.dart';
import 'package:pluto_grid/pluto_grid.dart';
import '../../models/database_connection.dart';
import '../../services/database/connection_service.dart';
import '../../services/database/database_service.dart';
import 'connection_dialog.dart';

/// A fully self-contained database workspace (like DBeaver).
/// Has its own sidebar (explorer tree) and main editor area.
class DatabaseWorkspace extends StatefulWidget {
  final String projectPath;

  const DatabaseWorkspace({super.key, required this.projectPath});

  @override
  State<DatabaseWorkspace> createState() => _DatabaseWorkspaceState();
}

// ─── Tree Node Model ─────────────────────────────────────────────────────────

class _DbNode {
  final String label;
  final _NodeType type;
  final DatabaseConnection connection;
  final String? database;
  final String? schema;
  final String? table;
  bool isExpanded;
  List<_DbNode>? children;
  bool isLoading;

  _DbNode({
    required this.label,
    required this.type,
    required this.connection,
    this.database,
    this.schema,
    this.table,
    this.isExpanded = false,
    this.children,
    this.isLoading = false,
  });
}

enum _NodeType { connection, database, schema, table }

// ─── Open Tab Model ──────────────────────────────────────────────────────────

class _DbTab {
  final String id;
  final String title;
  final _DbTabType type;
  final DatabaseConnection connection;
  final String? database;
  final String? schema;
  final String? table;

  _DbTab({
    required this.id,
    required this.title,
    required this.type,
    required this.connection,
    this.database,
    this.schema,
    this.table,
  });
}

enum _DbTabType { tableData, tableStructure, query, databaseInfo, queryHistory }

/// Helper class for the Create Table dialog
class _ColumnDef {
  final TextEditingController nameCtrl = TextEditingController();
  final TextEditingController typeCtrl = TextEditingController();
  bool isPrimaryKey = false;
  bool isNotNull = false;
}

class _DatabaseWorkspaceState extends State<DatabaseWorkspace> {
  // ── Sidebar State ──
  List<_DbNode> _roots = [];
  bool _isSidebarLoading = true;
  bool _sidebarVisible = true;

  // ── Tab State ──
  final List<_DbTab> _openTabs = [];
  int _activeTabIndex = -1;

  // ── Per-tab data cache ──
  final Map<String, QueryResult?> _dataCache = {};
  final Map<String, List<Map<String, dynamic>>?> _structureCache = {};
  final Map<String, List<String>?> _infoCache = {};
  final Map<String, bool> _loadingState = {};
  final Map<String, String?> _errorState = {};
  final Map<String, int> _pageState = {};

  // ── Query tab state ──
  final Map<String, TextEditingController> _queryControllers = {};
  final Map<String, FocusNode> _queryFocusNodes = {};
  final Map<String, List<String>> _querySuggestions = {};
  final Map<String, QueryResult?> _queryResults = {};
  final Map<String, String?> _queryErrors = {};
  final Map<String, bool> _queryExecuting = {};

  // ── Selected row for CRUD ──
  final Map<String, int> _selectedRowIndex = {};
  final Map<String, Map<String, dynamic>> _selectedRowData = {};

  @override
  void initState() {
    super.initState();
    _loadConnections();
  }

  @override
  void dispose() {
    for (final c in _queryControllers.values) {
      c.dispose();
    }
    for (final fn in _queryFocusNodes.values) {
      fn.dispose();
    }
    super.dispose();
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // SIDEBAR: Connection & Tree Logic
  // ═══════════════════════════════════════════════════════════════════════════

  Future<void> _loadConnections() async {
    setState(() => _isSidebarLoading = true);
    final conns = await ConnectionService.getConnections();
    setState(() {
      _roots = conns
          .map(
            (c) => _DbNode(
              label: c.name,
              type: _NodeType.connection,
              connection: c,
            ),
          )
          .toList();
      _isSidebarLoading = false;
    });
  }

  Future<void> _addConnection() async {
    final result = await showDialog<DatabaseConnection>(
      context: context,
      builder: (ctx) => ConnectionDialog(projectPath: widget.projectPath),
    );
    if (result != null) {
      await ConnectionService.saveConnection(result);
      _loadConnections();
    }
  }

  Future<void> _editConnection(_DbNode node) async {
    final result = await showDialog<DatabaseConnection>(
      context: context,
      builder: (ctx) => ConnectionDialog(
        projectPath: widget.projectPath,
        connection: node.connection,
      ),
    );
    if (result != null) {
      await ConnectionService.saveConnection(result);
      _loadConnections();
    }
  }

  Future<void> _deleteConnection(_DbNode node) async {
    final ok = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: const Color(0xFF252526),
        title: const Text(
          'Delete Connection',
          style: TextStyle(color: Colors.white),
        ),
        content: Text(
          'Delete "${node.connection.name}"?',
          style: const TextStyle(color: Colors.white70),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx, false),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () => Navigator.pop(ctx, true),
            child: const Text('Delete', style: TextStyle(color: Colors.red)),
          ),
        ],
      ),
    );
    if (ok == true) {
      await ConnectionService.deleteConnection(node.connection.id);
      _loadConnections();
    }
  }

  Future<void> _toggleNode(_DbNode node) async {
    if (node.type == _NodeType.table) {
      _openTableTab(node);
      return;
    }

    // For connection/database/schema nodes, open info tab instead of query tab
    if ((node.type == _NodeType.connection ||
            node.type == _NodeType.database ||
            node.type == _NodeType.schema) &&
        !node.isExpanded) {
      _openDatabaseInfoTab(node);
    }

    setState(() => node.isExpanded = !node.isExpanded);
    if (node.isExpanded && node.children == null) {
      await _loadChildren(node);
    }
  }

  Future<void> _loadChildren(_DbNode node) async {
    setState(() => node.isLoading = true);
    try {
      final svc = DatabaseService.fromConnection(node.connection);
      await svc.connect();

      List<_DbNode> children = [];

      switch (node.type) {
        case _NodeType.connection:
          final dbs = await svc.getDatabases();
          children = dbs
              .map(
                (db) => _DbNode(
                  label: db,
                  type: _NodeType.database,
                  connection: node.connection,
                  database: db,
                ),
              )
              .toList();
          break;

        case _NodeType.database:
          if (node.connection.engine == DatabaseEngine.postgres) {
            final schemas = await svc.getSchemas(node.database!);
            children = schemas
                .map(
                  (s) => _DbNode(
                    label: s,
                    type: _NodeType.schema,
                    connection: node.connection,
                    database: node.database,
                    schema: s,
                  ),
                )
                .toList();
          } else {
            final tables = await svc.getTables(node.database!, '');
            children = tables
                .map(
                  (t) => _DbNode(
                    label: t,
                    type: _NodeType.table,
                    connection: node.connection,
                    database: node.database,
                    schema: '',
                    table: t,
                  ),
                )
                .toList();
          }
          break;

        case _NodeType.schema:
          final tables = await svc.getTables(node.database!, node.schema!);
          children = tables
              .map(
                (t) => _DbNode(
                  label: t,
                  type: _NodeType.table,
                  connection: node.connection,
                  database: node.database,
                  schema: node.schema,
                  table: t,
                ),
              )
              .toList();
          break;

        case _NodeType.table:
          break;
      }

      await svc.disconnect();
      if (mounted)
        setState(() {
          node.children = children;
          node.isLoading = false;
        });
    } catch (e) {
      if (mounted) {
        setState(() {
          node.isLoading = false;
          node.isExpanded = false;
        });
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text('Error: $e')));
      }
    }
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // TAB MANAGEMENT
  // ═══════════════════════════════════════════════════════════════════════════

  void _openTableTab(_DbNode node) {
    final tabId =
        'data:${node.connection.id}:${node.database}:${node.schema}:${node.table}';

    // Check if already open
    final existingIdx = _openTabs.indexWhere((t) => t.id == tabId);
    if (existingIdx != -1) {
      setState(() => _activeTabIndex = existingIdx);
      // Close sidebar on mobile
      _closeSidebarOnMobile();
      return;
    }

    final tab = _DbTab(
      id: tabId,
      title: node.table!,
      type: _DbTabType.tableData,
      connection: node.connection,
      database: node.database,
      schema: node.schema,
      table: node.table,
    );
    setState(() {
      _openTabs.add(tab);
      _activeTabIndex = _openTabs.length - 1;
    });
    _fetchSuggestionsForQueryTab(tab);
    _loadTableData(tab);
    _closeSidebarOnMobile();
  }

  Future<void> _fetchSuggestionsForQueryTab(_DbTab tab) async {
    try {
      final svc = DatabaseService.fromConnection(tab.connection);
      await svc.connect();
      List<String> tables = [];
      if (tab.connection.engine == DatabaseEngine.sqlite) {
        tables = await svc.getTables('', '');
      } else if (tab.connection.engine == DatabaseEngine.mysql) {
        if (tab.database != null) {
          tables = await svc.getTables(tab.database!, '');
        }
      } else if (tab.connection.engine == DatabaseEngine.postgres) {
        if (tab.schema != null) {
          tables = await svc.getTables(tab.database!, tab.schema!);
        }
      }

      final Set<String> suggestions = {...tables};
      for (final struct in _structureCache.values) {
        if (struct != null) {
          for (final colDef in struct) {
            final colName =
                colDef['name']?.toString() ??
                colDef['Field']?.toString() ??
                colDef['column_name']?.toString() ??
                '';
            if (colName.isNotEmpty) suggestions.add(colName);
          }
        }
      }

      if (mounted) {
        setState(() {
          _querySuggestions[tab.id] = suggestions.toList();
        });
      }
      await svc.disconnect();
    } catch (_) {}
  }

  void _openQueryHistoryTab() {
    final id = 'history:global';
    final idx = _openTabs.indexWhere((t) => t.id == id);
    if (idx != -1) {
      setState(() => _activeTabIndex = idx);
      return;
    }
    // We need a dummy connection for the tab, but history doesn't strictly need one
    // Let's just use the first connection available, or a dummy if none.
    // Wait, the tab requires a connection.
    if (_roots.isEmpty) return;
    
    final newTab = _DbTab(
      id: id,
      title: 'History',
      type: _DbTabType.queryHistory,
      connection: _roots.first.connection,
    );
    setState(() {
      _openTabs.add(newTab);
      _activeTabIndex = _openTabs.length - 1;
    });
  }

  void _openQueryTab(DatabaseConnection conn, String? db, String? schema) {
    final tabId = 'query:${conn.id}:${DateTime.now().millisecondsSinceEpoch}';
    final tab = _DbTab(
      id: tabId,
      title: 'Query - ${conn.name}',
      type: _DbTabType.query,
      connection: conn,
      database: db,
      schema: schema,
    );
    _queryControllers.putIfAbsent(tabId, () => CodeController(language: sql));
    setState(() {
      _openTabs.add(tab);
      _activeTabIndex = _openTabs.length - 1;
    });
    _fetchSuggestionsForQueryTab(tab);
    _closeSidebarOnMobile();
  }

  void _openDatabaseInfoTab(_DbNode node) {
    final tabId =
        'info:${node.connection.id}:${node.database ?? ''}:${node.schema ?? ''}';
    final existingIdx = _openTabs.indexWhere((t) => t.id == tabId);
    if (existingIdx != -1) {
      setState(() => _activeTabIndex = existingIdx);
      _closeSidebarOnMobile();
      return;
    }

    final title = node.schema ?? node.database ?? node.connection.name;
    final tab = _DbTab(
      id: tabId,
      title: title,
      type: _DbTabType.databaseInfo,
      connection: node.connection,
      database: node.database,
      schema: node.schema,
    );
    setState(() {
      _openTabs.add(tab);
      _activeTabIndex = _openTabs.length - 1;
    });
    _fetchSuggestionsForQueryTab(tab);
    _loadDatabaseInfo(tab);
    _closeSidebarOnMobile();
  }

  void _closeTab(int index) {
    final tab = _openTabs[index];
    _dataCache.remove(tab.id);
    _structureCache.remove(tab.id);
    _loadingState.remove(tab.id);
    _errorState.remove(tab.id);
    _queryControllers[tab.id]?.dispose();
    _queryControllers.remove(tab.id);
    _queryResults.remove(tab.id);
    _queryErrors.remove(tab.id);
    _queryExecuting.remove(tab.id);

    setState(() {
      _openTabs.removeAt(index);
      if (_activeTabIndex >= _openTabs.length) {
        _activeTabIndex = _openTabs.length - 1;
      }
    });
  }

  void _closeSidebarOnMobile() {
    if (MediaQuery.of(context).size.width < 600) {
      setState(() => _sidebarVisible = false);
    }
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // DATA LOADING
  // ═══════════════════════════════════════════════════════════════════════════

  Future<void> _loadTableData(_DbTab tab) async {
    setState(() {
      _loadingState[tab.id] = true;
      _errorState[tab.id] = null;
    });
    try {
      final svc = DatabaseService.fromConnection(tab.connection);
      await svc.connect();

      final page = _pageState[tab.id] ?? 0;
      final offset = page * 100;
      String q;
      if (tab.connection.engine == DatabaseEngine.postgres) {
        q = 'SELECT * FROM "${tab.schema}"."${tab.table}" LIMIT 100 OFFSET $offset';
      } else if (tab.connection.engine == DatabaseEngine.mysql) {
        q = 'SELECT * FROM `${tab.database}`.`${tab.table}` LIMIT 100 OFFSET $offset';
      } else {
        q = 'SELECT * FROM "${tab.table}" LIMIT 100 OFFSET $offset';
      }

      final res = await svc.executeQuery(q);
      await svc.disconnect();
      if (mounted) {
        setState(() {
          _dataCache[tab.id] = res;
          _loadingState[tab.id] = false;
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _errorState[tab.id] = e.toString();
          _loadingState[tab.id] = false;
        });
      }
    }
  }

  Future<void> _loadTableStructure(_DbTab tab) async {
    final structId = 'struct:${tab.id}';
    setState(() {
      _loadingState[structId] = true;
      _errorState[structId] = null;
    });
    try {
      final svc = DatabaseService.fromConnection(tab.connection);
      await svc.connect();
      final res = await svc.getTableStructure(
        tab.database ?? '',
        tab.schema ?? '',
        tab.table!,
      );
      await svc.disconnect();
      if (mounted) {
        setState(() {
          _structureCache[tab.id] = res;
          _loadingState[structId] = false;
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _errorState[structId] = e.toString();
          _loadingState[structId] = false;
        });
      }
    }
  }

  Future<void> _loadDatabaseInfo(_DbTab tab) async {
    setState(() => _loadingState[tab.id] = true);
    try {
      final svc = DatabaseService.fromConnection(tab.connection);
      await svc.connect();
      List<String> items = [];
      if (tab.connection.engine == DatabaseEngine.sqlite) {
        items = await svc.getTables('', '');
      } else if (tab.connection.engine == DatabaseEngine.mysql) {
        if (tab.database != null) {
          items = await svc.getTables(tab.database!, '');
        } else {
          items = await svc.getDatabases();
        }
      } else if (tab.connection.engine == DatabaseEngine.postgres) {
        if (tab.schema != null) {
          items = await svc.getTables(tab.database!, tab.schema!);
        } else if (tab.database != null) {
          items = await svc.getSchemas(tab.database!);
        } else {
          items = await svc.getDatabases();
        }
      }
      await svc.disconnect();
      setState(() {
        _infoCache[tab.id] = items;
        _errorState.remove(tab.id);
      });
    } catch (e) {
      setState(() => _errorState[tab.id] = e.toString());
    } finally {
      setState(() => _loadingState[tab.id] = false);
    }
  }

  Future<void> _executeQuery(_DbTab tab) async {
    final ctrl = _queryControllers[tab.id];
    if (ctrl == null) return;
    final query = ctrl.text.trim();
    if (query.isEmpty) return;

    setState(() {
      _queryExecuting[tab.id] = true;
      _queryErrors[tab.id] = null;
    });

    try {
      final svc = DatabaseService.fromConnection(tab.connection);
      await svc.connect();

      if (tab.connection.engine == DatabaseEngine.mysql &&
          tab.database != null) {
        await svc.executeQuery('USE `${tab.database}`');
      } else if (tab.connection.engine == DatabaseEngine.postgres &&
          tab.schema != null) {
        await svc.executeQuery('SET search_path TO "${tab.schema}"');
      }

      final res = await svc.executeQuery(query);

      QueryHistoryService.addHistory(QueryHistoryItem(
        query: query,
        executedAt: DateTime.now(),
        connectionName: tab.connection.name,
        database: tab.database,
      ));

      await svc.disconnect();

      if (mounted) {
        setState(() {
          _queryResults[tab.id] = res;
          _queryExecuting[tab.id] = false;
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _queryErrors[tab.id] = e.toString();
          _queryExecuting[tab.id] = false;
        });
      }
    }
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // CRUD OPERATIONS
  // ═══════════════════════════════════════════════════════════════════════════

  /// Helper to run a mutation query and show result
  Future<bool> _runMutation(
    DatabaseConnection conn,
    String? db,
    String? schema,
    String query,
  ) async {
    try {
      final svc = DatabaseService.fromConnection(conn);
      await svc.connect();
      if (conn.engine == DatabaseEngine.mysql && db != null) {
        await svc.executeQuery('USE `$db`');
      } else if (conn.engine == DatabaseEngine.postgres && schema != null) {
        await svc.executeQuery('SET search_path TO "$schema"');
      }
      final res = await svc.executeQuery(query);

      QueryHistoryService.addHistory(QueryHistoryItem(
        query: query,
        executedAt: DateTime.now(),
        connectionName: conn.name,
        database: db,
      ));

      await svc.disconnect();
      if (res.error != null && res.error!.isNotEmpty) {
        if (mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              content: Text('Error: ${res.error}'),
              backgroundColor: Colors.red,
            ),
          );
        }
        return false;
      }
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Success. Affected rows: ${res.affectedRows}'),
            backgroundColor: const Color(0xFF007ACC),
          ),
        );
      }
      return true;
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Error: $e'), backgroundColor: Colors.red),
        );
      }
      return false;
    }
  }

  // ── CREATE TABLE ──
  Future<void> _showCreateTableDialog(
    DatabaseConnection conn,
    String? db,
    String? schema,
  ) async {
    final tableNameCtrl = TextEditingController();
    final columns = <_ColumnDef>[_ColumnDef()];

    final result = await showDialog<bool>(
      context: context,
      builder: (ctx) => StatefulBuilder(
        builder: (ctx, setDialogState) => AlertDialog(
          backgroundColor: const Color(0xFF252526),
          title: const Text(
            'Create Table',
            style: TextStyle(color: Colors.white),
          ),
          content: SizedBox(
            width: 500,
            child: SingleChildScrollView(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  // Table name
                  const Text(
                    'Table Name',
                    style: TextStyle(color: Colors.white70, fontSize: 12),
                  ),
                  const SizedBox(height: 4),
                  TextField(
                    controller: tableNameCtrl,
                    style: const TextStyle(color: Colors.white, fontSize: 14),
                    decoration: const InputDecoration(
                      isDense: true,
                      filled: true,
                      fillColor: Color(0xFF1E1E1E),
                      border: OutlineInputBorder(borderSide: BorderSide.none),
                      hintText: 'e.g. users',
                      hintStyle: TextStyle(color: Colors.white24),
                    ),
                  ),
                  const SizedBox(height: 16),
                  // Columns header
                  Row(
                    children: [
                      const Text(
                        'Columns',
                        style: TextStyle(color: Colors.white70, fontSize: 12),
                      ),
                      const Spacer(),
                      TextButton.icon(
                        onPressed: () =>
                            setDialogState(() => columns.add(_ColumnDef())),
                        icon: const Icon(Icons.add, size: 14),
                        label: const Text(
                          'Add Column',
                          style: TextStyle(fontSize: 12),
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 4),
                  // Column list
                  ...columns.asMap().entries.map((entry) {
                    final i = entry.key;
                    final col = entry.value;
                    return Padding(
                      padding: const EdgeInsets.only(bottom: 8),
                      child: Row(
                        children: [
                          // Name
                          Expanded(
                            flex: 3,
                            child: TextField(
                              controller: col.nameCtrl,
                              style: const TextStyle(
                                color: Colors.white,
                                fontSize: 13,
                              ),
                              decoration: InputDecoration(
                                isDense: true,
                                filled: true,
                                fillColor: const Color(0xFF1E1E1E),
                                border: const OutlineInputBorder(
                                  borderSide: BorderSide.none,
                                ),
                                hintText: 'col_name',
                                hintStyle: const TextStyle(
                                  color: Colors.white24,
                                  fontSize: 13,
                                ),
                                contentPadding: const EdgeInsets.symmetric(
                                  horizontal: 8,
                                  vertical: 8,
                                ),
                              ),
                            ),
                          ),
                          const SizedBox(width: 4),
                          // Type
                          Expanded(
                            flex: 2,
                            child: TextField(
                              controller: col.typeCtrl,
                              style: const TextStyle(
                                color: Colors.white,
                                fontSize: 13,
                              ),
                              decoration: InputDecoration(
                                isDense: true,
                                filled: true,
                                fillColor: const Color(0xFF1E1E1E),
                                border: const OutlineInputBorder(
                                  borderSide: BorderSide.none,
                                ),
                                hintText: 'TEXT',
                                hintStyle: const TextStyle(
                                  color: Colors.white24,
                                  fontSize: 13,
                                ),
                                contentPadding: const EdgeInsets.symmetric(
                                  horizontal: 8,
                                  vertical: 8,
                                ),
                              ),
                            ),
                          ),
                          const SizedBox(width: 4),
                          // PK checkbox
                          Tooltip(
                            message: 'Primary Key',
                            child: Checkbox(
                              value: col.isPrimaryKey,
                              onChanged: (v) => setDialogState(
                                () => col.isPrimaryKey = v ?? false,
                              ),
                              materialTapTargetSize:
                                  MaterialTapTargetSize.shrinkWrap,
                              visualDensity: VisualDensity.compact,
                            ),
                          ),
                          const Text(
                            'PK',
                            style: TextStyle(
                              color: Colors.white54,
                              fontSize: 10,
                            ),
                          ),
                          // Not Null checkbox
                          Tooltip(
                            message: 'NOT NULL',
                            child: Checkbox(
                              value: col.isNotNull,
                              onChanged: (v) => setDialogState(
                                () => col.isNotNull = v ?? false,
                              ),
                              materialTapTargetSize:
                                  MaterialTapTargetSize.shrinkWrap,
                              visualDensity: VisualDensity.compact,
                            ),
                          ),
                          const Text(
                            'NN',
                            style: TextStyle(
                              color: Colors.white54,
                              fontSize: 10,
                            ),
                          ),
                          // Remove
                          if (columns.length > 1)
                            IconButton(
                              icon: const Icon(
                                Icons.close,
                                size: 14,
                                color: Colors.red,
                              ),
                              onPressed: () =>
                                  setDialogState(() => columns.removeAt(i)),
                              padding: EdgeInsets.zero,
                              constraints: const BoxConstraints(
                                minWidth: 24,
                                minHeight: 24,
                              ),
                            ),
                        ],
                      ),
                    );
                  }),
                ],
              ),
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(ctx, false),
              child: const Text(
                'Cancel',
                style: TextStyle(color: Colors.white54),
              ),
            ),
            ElevatedButton(
              onPressed: () => Navigator.pop(ctx, true),
              style: ElevatedButton.styleFrom(
                backgroundColor: const Color(0xFF007ACC),
              ),
              child: const Text(
                'Create',
                style: TextStyle(color: Colors.white),
              ),
            ),
          ],
        ),
      ),
    );

    if (result != true) return;
    final tableName = tableNameCtrl.text.trim();
    if (tableName.isEmpty ||
        columns.every((c) => c.nameCtrl.text.trim().isEmpty))
      return;

    final colDefs = columns
        .where((c) => c.nameCtrl.text.trim().isNotEmpty)
        .map((c) {
          final name = c.nameCtrl.text.trim();
          final type = c.typeCtrl.text.trim().isEmpty
              ? 'TEXT'
              : c.typeCtrl.text.trim();
          final pk = c.isPrimaryKey ? ' PRIMARY KEY' : '';
          final nn = c.isNotNull ? ' NOT NULL' : '';
          return '"$name" $type$pk$nn';
        })
        .join(', ');

    final sql = 'CREATE TABLE "$tableName" ($colDefs)';
    final ok = await _runMutation(conn, db, schema, sql);
    if (ok) _refreshSidebarNode(conn, db, schema);
  }

  // ── INSERT ROW ──
  Future<void> _showInsertRowDialog(_DbTab tab) async {
    final data = _dataCache[tab.id];
    List<String> columns;
    if (data != null && data.columns.isNotEmpty) {
      columns = data.columns;
    } else {
      // Fetch structure to get column names
      try {
        final svc = DatabaseService.fromConnection(tab.connection);
        await svc.connect();
        final struct = await svc.getTableStructure(
          tab.database ?? '',
          tab.schema ?? '',
          tab.table!,
        );
        await svc.disconnect();
        columns = struct
            .map(
              (s) => s['name']?.toString() ?? s.values.first?.toString() ?? '',
            )
            .where((s) => s.isNotEmpty)
            .toList();
      } catch (_) {
        columns = [];
      }
    }
    if (columns.isEmpty) {
      if (mounted)
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('Cannot determine columns')),
        );
      return;
    }

    final controllers = {for (final c in columns) c: TextEditingController()};

    final result = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: const Color(0xFF252526),
        title: Text(
          'Insert into ${tab.table}',
          style: const TextStyle(color: Colors.white),
        ),
        content: SizedBox(
          width: 400,
          child: SingleChildScrollView(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: columns.map((col) {
                return Padding(
                  padding: const EdgeInsets.only(bottom: 8),
                  child: Row(
                    children: [
                      SizedBox(
                        width: 120,
                        child: Text(
                          col,
                          style: const TextStyle(
                            color: Colors.white70,
                            fontSize: 12,
                          ),
                        ),
                      ),
                      Expanded(
                        child: TextField(
                          controller: controllers[col],
                          style: const TextStyle(
                            color: Colors.white,
                            fontSize: 13,
                          ),
                          decoration: InputDecoration(
                            isDense: true,
                            filled: true,
                            fillColor: const Color(0xFF1E1E1E),
                            border: const OutlineInputBorder(
                              borderSide: BorderSide.none,
                            ),
                            hintText: 'NULL',
                            hintStyle: const TextStyle(
                              color: Colors.white24,
                              fontSize: 13,
                            ),
                            contentPadding: const EdgeInsets.symmetric(
                              horizontal: 8,
                              vertical: 8,
                            ),
                          ),
                        ),
                      ),
                    ],
                  ),
                );
              }).toList(),
            ),
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx, false),
            child: const Text(
              'Cancel',
              style: TextStyle(color: Colors.white54),
            ),
          ),
          ElevatedButton(
            onPressed: () => Navigator.pop(ctx, true),
            style: ElevatedButton.styleFrom(
              backgroundColor: const Color(0xFF007ACC),
            ),
            child: const Text('Insert', style: TextStyle(color: Colors.white)),
          ),
        ],
      ),
    );

    if (result != true) return;

    final colNames = <String>[];
    final colValues = <String>[];
    for (final col in columns) {
      final val = controllers[col]!.text;
      if (val.isNotEmpty) {
        colNames.add('"$col"');
        colValues.add("'${val.replaceAll("'", "''")}'");
      }
    }
    if (colNames.isEmpty) return;

    final sql =
        'INSERT INTO "${tab.table}" (${colNames.join(', ')}) VALUES (${colValues.join(', ')})';
    final ok = await _runMutation(
      tab.connection,
      tab.database,
      tab.schema,
      sql,
    );
    if (ok) _loadTableData(tab);
  }

  // ── EDIT ROW ──
  Future<void> _showEditRowDialog(_DbTab tab) async {
    final rowData = _selectedRowData[tab.id];
    if (rowData == null) return;

    final data = _dataCache[tab.id];
    if (data == null) return;
    final columns = data.columns;

    final controllers = {
      for (final c in columns)
        c: TextEditingController(text: rowData[c]?.toString() ?? ''),
    };

    final result = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: const Color(0xFF252526),
        title: Text(
          'Edit Row in ${tab.table}',
          style: const TextStyle(color: Colors.white),
        ),
        content: SizedBox(
          width: 400,
          child: SingleChildScrollView(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: columns.map((col) {
                return Padding(
                  padding: const EdgeInsets.only(bottom: 8),
                  child: Row(
                    children: [
                      SizedBox(
                        width: 120,
                        child: Text(
                          col,
                          style: const TextStyle(
                            color: Colors.white70,
                            fontSize: 12,
                          ),
                        ),
                      ),
                      Expanded(
                        child: TextField(
                          controller: controllers[col],
                          style: const TextStyle(
                            color: Colors.white,
                            fontSize: 13,
                          ),
                          decoration: InputDecoration(
                            isDense: true,
                            filled: true,
                            fillColor: const Color(0xFF1E1E1E),
                            border: const OutlineInputBorder(
                              borderSide: BorderSide.none,
                            ),
                            contentPadding: const EdgeInsets.symmetric(
                              horizontal: 8,
                              vertical: 8,
                            ),
                          ),
                        ),
                      ),
                    ],
                  ),
                );
              }).toList(),
            ),
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx, false),
            child: const Text(
              'Cancel',
              style: TextStyle(color: Colors.white54),
            ),
          ),
          ElevatedButton(
            onPressed: () => Navigator.pop(ctx, true),
            style: ElevatedButton.styleFrom(
              backgroundColor: const Color(0xFF007ACC),
            ),
            child: const Text('Update', style: TextStyle(color: Colors.white)),
          ),
        ],
      ),
    );

    if (result != true) return;

    // Build SET clause from changed values
    final setClauses = <String>[];
    for (final col in columns) {
      final newVal = controllers[col]!.text;
      final oldVal = rowData[col]?.toString() ?? '';
      if (newVal != oldVal) {
        setClauses.add('"$col" = \'${newVal.replaceAll("'", "''")}\'');
      }
    }
    if (setClauses.isEmpty) return;

    // Build WHERE clause from original row
    final whereClauses = columns
        .map((col) {
          final val = rowData[col];
          if (val == null) return '"$col" IS NULL';
          return '"$col" = \'${val.toString().replaceAll("'", "''")}\'';
        })
        .join(' AND ');

    final sql =
        'UPDATE "${tab.table}" SET ${setClauses.join(', ')} WHERE $whereClauses';
    final ok = await _runMutation(
      tab.connection,
      tab.database,
      tab.schema,
      sql,
    );
    if (ok) _loadTableData(tab);
  }

  // ── DELETE ROW ──
  Future<void> _deleteSelectedRow(_DbTab tab) async {
    final rowData = _selectedRowData[tab.id];
    if (rowData == null) return;

    final data = _dataCache[tab.id];
    if (data == null) return;

    final confirm = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: const Color(0xFF252526),
        title: const Text('Delete Row', style: TextStyle(color: Colors.white)),
        content: const Text(
          'Are you sure you want to delete this row?',
          style: TextStyle(color: Colors.white70),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx, false),
            child: const Text(
              'Cancel',
              style: TextStyle(color: Colors.white54),
            ),
          ),
          ElevatedButton(
            onPressed: () => Navigator.pop(ctx, true),
            style: ElevatedButton.styleFrom(backgroundColor: Colors.red),
            child: const Text('Delete', style: TextStyle(color: Colors.white)),
          ),
        ],
      ),
    );
    if (confirm != true) return;

    final whereClauses = data.columns
        .map((col) {
          final val = rowData[col];
          if (val == null) return '"$col" IS NULL';
          return '"$col" = \'${val.toString().replaceAll("'", "''")}\'';
        })
        .join(' AND ');

    final sql = 'DELETE FROM "${tab.table}" WHERE $whereClauses LIMIT 1';
    final ok = await _runMutation(
      tab.connection,
      tab.database,
      tab.schema,
      sql,
    );
    if (ok) {
      _selectedRowData.remove(tab.id);
      _selectedRowIndex.remove(tab.id);
      _loadTableData(tab);
    }
  }

  // ── DROP TABLE ──
  Future<void> _dropTable(_DbNode node) async {
    final confirm = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: const Color(0xFF252526),
        title: const Text('Drop Table', style: TextStyle(color: Colors.white)),
        content: Text(
          'Are you sure you want to DROP table "${node.table}"?\nThis action cannot be undone.',
          style: const TextStyle(color: Colors.white70),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx, false),
            child: const Text(
              'Cancel',
              style: TextStyle(color: Colors.white54),
            ),
          ),
          ElevatedButton(
            onPressed: () => Navigator.pop(ctx, true),
            style: ElevatedButton.styleFrom(backgroundColor: Colors.red),
            child: const Text(
              'Drop Table',
              style: TextStyle(color: Colors.white),
            ),
          ),
        ],
      ),
    );
    if (confirm != true) return;

    final sql = 'DROP TABLE "${node.table}"';
    final ok = await _runMutation(
      node.connection,
      node.database,
      node.schema,
      sql,
    );
    if (ok) _refreshSidebarNode(node.connection, node.database, node.schema);
  }

  /// Refresh the relevant parent node in the sidebar tree after table creation/drop
  void _refreshSidebarNode(
    DatabaseConnection conn,
    String? db,
    String? schema,
  ) {
    for (final root in _roots) {
      if (root.connection.id == conn.id) {
        if (root.children != null) {
          for (final dbNode in root.children!) {
            if (dbNode.database == db) {
              if (schema != null &&
                  schema.isNotEmpty &&
                  dbNode.children != null) {
                for (final schemaNode in dbNode.children!) {
                  if (schemaNode.schema == schema) {
                    schemaNode.children = null;
                    schemaNode.isExpanded = false;
                    _toggleNode(schemaNode);
                    return;
                  }
                }
              }
              dbNode.children = null;
              dbNode.isExpanded = false;
              _toggleNode(dbNode);
              return;
            }
          }
        }
        root.children = null;
        root.isExpanded = false;
        _toggleNode(root);
        return;
      }
    }
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // BUILD
  // ═══════════════════════════════════════════════════════════════════════════

  @override
  Widget build(BuildContext context) {
    final isMobile = MediaQuery.of(context).size.width < 600;

    return Container(
      color: const Color(0xFF1E1E1E),
      child: Row(
        children: [
          // ── Desktop sidebar ──
          if (!isMobile && _sidebarVisible) _buildSidebar(),
          // ── Main area ──
          Expanded(
            child: Stack(
              children: [
                Column(
                  children: [
                    // Toolbar
                    _buildToolbar(isMobile),
                    // Tab bar
                    _buildTabBar(),
                    // Tab content
                    Expanded(child: _buildTabContent()),
                  ],
                ),
                // ── Mobile overlay sidebar ──
                if (isMobile && _sidebarVisible) ...[
                  Positioned.fill(
                    child: GestureDetector(
                      onTap: () => setState(() => _sidebarVisible = false),
                      behavior: HitTestBehavior.opaque,
                      child: Container(color: Colors.black54),
                    ),
                  ),
                  Positioned(
                    left: 0,
                    top: 0,
                    bottom: 0,
                    width: 280,
                    child: Material(
                      elevation: 16,
                      color: Colors.transparent,
                      child: _buildSidebar(),
                    ),
                  ),
                ],
              ],
            ),
          ),
        ],
      ),
    );
  }

  // ── Toolbar ──────────────────────────────────────────────────────────────

  Widget _buildToolbar(bool isMobile) {
    return Container(
      height: 40,
      color: const Color(0xFF2D2D30),
      padding: const EdgeInsets.symmetric(horizontal: 8),
      child: Row(
        children: [
          // Toggle sidebar
          IconButton(
            icon: Icon(
              _sidebarVisible ? Icons.menu_open : Icons.menu,
              size: 18,
              color: Colors.white70,
            ),
            onPressed: () => setState(() => _sidebarVisible = !_sidebarVisible),
            tooltip: 'Toggle Sidebar',
            padding: EdgeInsets.zero,
            constraints: const BoxConstraints(minWidth: 32, minHeight: 32),
          ),
          const SizedBox(width: 4),
          // New query button
          IconButton(
            icon: const Icon(Icons.code, size: 18, color: Colors.white70),
            onPressed: _openNewQueryTab,
            tooltip: 'New SQL Query',
            padding: EdgeInsets.zero,
            constraints: const BoxConstraints(minWidth: 32, minHeight: 32),
          ),
          const Spacer(),
          // Connection info
          if (_activeTabIndex >= 0 && _activeTabIndex < _openTabs.length)
            Text(
              _openTabs[_activeTabIndex].connection.name,
              style: const TextStyle(color: Colors.white38, fontSize: 12),
            ),
        ],
      ),
    );
  }

  void _openNewQueryTab() {
    // Use the first connection if available
    if (_roots.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('No connections available. Add one first.'),
        ),
      );
      return;
    }
    final conn = _roots.first.connection;
    _openQueryTab(conn, null, null);
  }

  // ── Sidebar ──────────────────────────────────────────────────────────────

  Widget _buildSidebar() {
    return Container(
      width: 240,
      decoration: const BoxDecoration(
        color: Color(0xFF252526),
        border: Border(right: BorderSide(color: Color(0xFF333333), width: 1)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          // Header
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
            color: const Color(0xFF2D2D30),
            child: Row(
              children: [
                const Icon(
                  Icons.dns_outlined,
                  size: 14,
                  color: Color(0xFF007ACC),
                ),
                const SizedBox(width: 6),
                const Expanded(
                  child: Text(
                    'DATABASE NAVIGATOR',
                    style: TextStyle(
                      color: Colors.white,
                      fontSize: 11,
                      fontWeight: FontWeight.bold,
                      letterSpacing: 0.5,
                    ),
                  ),
                ),
                _sidebarAction(Icons.add, 'New Connection', _addConnection),
                const SizedBox(width: 4),
                _sidebarAction(Icons.refresh, 'Refresh', _loadConnections),
              ],
            ),
          ),
          // Tree
          if (_isSidebarLoading)
            const Expanded(
              child: Center(
                child: SizedBox(
                  width: 20,
                  height: 20,
                  child: CircularProgressIndicator(strokeWidth: 2),
                ),
              ),
            )
          else if (_roots.isEmpty)
            Expanded(
              child: Center(
                child: Padding(
                  padding: const EdgeInsets.all(24),
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      const Icon(
                        Icons.dns_outlined,
                        size: 48,
                        color: Colors.white24,
                      ),
                      const SizedBox(height: 12),
                      const Text(
                        'No Connections',
                        style: TextStyle(color: Colors.white54, fontSize: 14),
                      ),
                      const SizedBox(height: 8),
                      ElevatedButton.icon(
                        onPressed: _addConnection,
                        icon: const Icon(Icons.add, size: 16),
                        label: const Text('Add Connection'),
                        style: ElevatedButton.styleFrom(
                          backgroundColor: const Color(0xFF007ACC),
                          foregroundColor: Colors.white,
                        ),
                      ),
                    ],
                  ),
                ),
              ),
            )
          else
            Expanded(
              child: ListView(
                padding: const EdgeInsets.symmetric(vertical: 4),
                children: _roots.map((n) => _buildTreeNode(n, 0)).toList(),
              ),
            ),
        ],
      ),
    );
  }

  Widget _sidebarAction(IconData icon, String tooltip, VoidCallback onTap) {
    return Tooltip(
      message: tooltip,
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(4),
        child: Padding(
          padding: const EdgeInsets.all(2),
          child: Icon(icon, size: 14, color: Colors.white70),
        ),
      ),
    );
  }

  Widget _buildTreeNode(_DbNode node, int depth) {
    final isConn = node.type == _NodeType.connection;
    final isTable = node.type == _NodeType.table;
    final isSchema = node.type == _NodeType.schema;
    final isDb = node.type == _NodeType.database;

    IconData icon;
    Color iconColor;
    if (isConn) {
      icon = Icons.storage;
      iconColor = const Color(0xFF007ACC);
    } else if (isDb) {
      icon = Icons.dataset;
      iconColor = const Color(0xFFDCDCAA);
    } else if (isSchema) {
      icon = Icons.folder_outlined;
      iconColor = const Color(0xFFE8AB53);
    } else {
      icon = Icons.table_chart_outlined;
      iconColor = const Color(0xFF4EC9B0);
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        InkWell(
          onTap: () => _toggleNode(node),
          onLongPress: (isConn || isDb || isSchema || isTable)
              ? () => _showNodeMenu(node)
              : null,
          hoverColor: const Color(0xFF2A2D2E),
          child: Container(
            padding: EdgeInsets.only(
              left: 8.0 + depth * 16.0,
              right: 4,
              top: 5,
              bottom: 5,
            ),
            child: Row(
              children: [
                if (!isTable)
                  SizedBox(
                    width: 16,
                    child: node.isLoading
                        ? const SizedBox(
                            width: 12,
                            height: 12,
                            child: CircularProgressIndicator(strokeWidth: 1.5),
                          )
                        : Icon(
                            node.isExpanded
                                ? Icons.expand_more
                                : Icons.chevron_right,
                            size: 16,
                            color: Colors.white38,
                          ),
                  )
                else
                  const SizedBox(width: 16),
                const SizedBox(width: 4),
                Icon(icon, size: 14, color: iconColor),
                const SizedBox(width: 6),
                Expanded(
                  child: Text(
                    node.label,
                    style: const TextStyle(
                      color: Color(0xFFCCCCCC),
                      fontSize: 13,
                    ),
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
                if (isConn)
                  Padding(
                    padding: const EdgeInsets.only(left: 4),
                    child: Text(
                      node.connection.engine.name.toUpperCase(),
                      style: const TextStyle(
                        color: Colors.white24,
                        fontSize: 9,
                      ),
                    ),
                  ),
              ],
            ),
          ),
        ),
        if (node.isExpanded && node.children != null)
          ...node.children!.map((child) => _buildTreeNode(child, depth + 1)),
      ],
    );
  }

  void _showNodeMenu(_DbNode node) {
    final items = <Widget>[];


    items.add(
      ListTile(
        leading: const Icon(Icons.history, color: Colors.white),
        title: const Text(
          'Query History',
          style: TextStyle(color: Colors.white),
        ),
        onTap: () {
          Navigator.pop(context);
          _openQueryHistoryTab();
        },
      ),
    );

    // All nodes: SQL Editor
    items.add(
      ListTile(
        leading: const Icon(Icons.code, color: Colors.white),
        title: const Text(
          'Open SQL Editor',
          style: TextStyle(color: Colors.white),
        ),
        onTap: () {
          Navigator.pop(context);
          _openQueryTab(node.connection, node.database, node.schema);
        },
      ),
    );

    // Database/Schema nodes: Create Table
    if (node.type == _NodeType.database || node.type == _NodeType.schema) {
      items.add(
        ListTile(
          leading: const Icon(Icons.add_box_outlined, color: Color(0xFF4EC9B0)),
          title: const Text(
            'Create Table',
            style: TextStyle(color: Colors.white),
          ),
          onTap: () {
            Navigator.pop(context);
            _showCreateTableDialog(node.connection, node.database, node.schema);
          },
        ),
      );
    }

    // Table nodes: View Data, Drop
    if (node.type == _NodeType.table) {
      items.add(
        ListTile(
          leading: const Icon(Icons.table_chart, color: Color(0xFF4EC9B0)),
          title: const Text('View Data', style: TextStyle(color: Colors.white)),
          onTap: () {
            Navigator.pop(context);
            _openTableTab(node);
          },
        ),
      );
      items.add(
        ListTile(
          leading: const Icon(Icons.delete_forever, color: Colors.red),
          title: const Text('Drop Table', style: TextStyle(color: Colors.red)),
          onTap: () {
            Navigator.pop(context);
            _dropTable(node);
          },
        ),
      );
    }

    // Connection nodes: Edit, Delete
    if (node.type == _NodeType.connection) {
      items.add(
        ListTile(
          leading: const Icon(Icons.edit, color: Colors.white),
          title: const Text(
            'Edit Connection',
            style: TextStyle(color: Colors.white),
          ),
          onTap: () {
            Navigator.pop(context);
            _editConnection(node);
          },
        ),
      );
      items.add(
        ListTile(
          leading: const Icon(Icons.delete, color: Colors.red),
          title: const Text(
            'Delete Connection',
            style: TextStyle(color: Colors.red),
          ),
          onTap: () {
            Navigator.pop(context);
            _deleteConnection(node);
          },
        ),
      );
    }

    // All nodes: Refresh
    items.add(
      ListTile(
        leading: const Icon(Icons.refresh, color: Colors.white),
        title: const Text('Refresh', style: TextStyle(color: Colors.white)),
        onTap: () {
          Navigator.pop(context);
          setState(() {
            node.children = null;
            node.isExpanded = false;
          });
          _toggleNode(node);
        },
      ),
    );

    showModalBottomSheet(
      context: context,
      backgroundColor: const Color(0xFF2D2D30),
      builder: (ctx) => Column(mainAxisSize: MainAxisSize.min, children: items),
    );
  }

  // ── Tab Bar ──────────────────────────────────────────────────────────────

  Widget _buildTabBar() {
    if (_openTabs.isEmpty) return const SizedBox.shrink();

    return Container(
      height: 35,
      color: const Color(0xFF252526),
      child: ListView.builder(
        scrollDirection: Axis.horizontal,
        itemCount: _openTabs.length,
        itemBuilder: (context, index) {
          final tab = _openTabs[index];
          final isActive = index == _activeTabIndex;

          IconData tabIcon;
          Color tabColor;
          if (tab.type == _DbTabType.query) {
            tabIcon = Icons.code;
            tabColor = const Color(0xFF007ACC);
          } else if (tab.type == _DbTabType.tableStructure) {
            tabIcon = Icons.view_column;
            tabColor = const Color(0xFFE8AB53);
          } else if (tab.type == _DbTabType.databaseInfo) {
            tabIcon = Icons.info_outline;
            tabColor = const Color(0xFF007ACC);
          } else {
            tabIcon = Icons.table_chart_outlined;
            tabColor = const Color(0xFF4EC9B0);
          }

          return GestureDetector(
            onTap: () => setState(() => _activeTabIndex = index),
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 12),
              decoration: BoxDecoration(
                color: isActive
                    ? const Color(0xFF1E1E1E)
                    : const Color(0xFF2D2D30),
                border: Border(
                  top: BorderSide(
                    color: isActive
                        ? const Color(0xFF007ACC)
                        : Colors.transparent,
                    width: 2,
                  ),
                  right: const BorderSide(color: Color(0xFF333333), width: 1),
                ),
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(tabIcon, size: 13, color: tabColor),
                  const SizedBox(width: 6),
                  Text(
                    tab.title,
                    style: TextStyle(
                      color: isActive ? Colors.white : Colors.white54,
                      fontSize: 12,
                    ),
                  ),
                  const SizedBox(width: 8),
                  InkWell(
                    onTap: () => _closeTab(index),
                    child: Icon(
                      Icons.close,
                      size: 14,
                      color: isActive ? Colors.white54 : Colors.white24,
                    ),
                  ),
                ],
              ),
            ),
          );
        },
      ),
    );
  }

  // ── Tab Content ──────────────────────────────────────────────────────────

  Widget _buildTabContent() {
    if (_openTabs.isEmpty || _activeTabIndex < 0) {
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.storage, size: 64, color: Colors.white12),
            const SizedBox(height: 16),
            const Text(
              'RPL Studio Database',
              style: TextStyle(
                color: Colors.white38,
                fontSize: 18,
                fontWeight: FontWeight.w300,
              ),
            ),
            const SizedBox(height: 8),
            Text(
              _roots.isEmpty
                  ? 'Add a connection to get started'
                  : 'Select a table or open a query from the sidebar',
              style: const TextStyle(color: Colors.white24, fontSize: 13),
            ),
          ],
        ),
      );
    }

    final tab = _openTabs[_activeTabIndex];

    switch (tab.type) {
      case _DbTabType.tableData:
        return _buildTableDataView(tab);
      case _DbTabType.tableStructure:
        return _buildTableStructureView(tab);
      case _DbTabType.query:
        return _buildQueryView(tab);
      case _DbTabType.databaseInfo:
        return _buildDatabaseInfoView(tab);
      case _DbTabType.queryHistory:
        return _buildQueryHistoryGrid(tab);
    }
  }

  // ── Database Info View ───────────────────────────────────────────────────

  Widget _buildDatabaseInfoView(_DbTab tab) {
    final loading = _loadingState[tab.id] ?? false;
    final error = _errorState[tab.id];
    final items = _infoCache[tab.id];

    final bool canCreateTable =
        (tab.connection.engine == DatabaseEngine.sqlite) ||
        (tab.connection.engine == DatabaseEngine.mysql &&
            tab.database != null) ||
        (tab.connection.engine == DatabaseEngine.postgres &&
            tab.schema != null);

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        // Toolbar
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          color: const Color(0xFF2D2D30),
          child: Row(
            children: [
              const Icon(Icons.info_outline, size: 16, color: Colors.white70),
              const SizedBox(width: 8),
              Text(
                'Info: ${tab.title}',
                style: const TextStyle(
                  color: Colors.white,
                  fontWeight: FontWeight.bold,
                ),
              ),
              const Spacer(),
              if (canCreateTable)
                ElevatedButton.icon(
                  onPressed: () => _showCreateTableDialog(
                    tab.connection,
                    tab.database,
                    tab.schema,
                  ),
                  icon: const Icon(Icons.add, size: 12),
                  label: const Text('Create Table'),
                  style: ElevatedButton.styleFrom(
                    backgroundColor: const Color(0xFF4EC9B0),
                    foregroundColor: Colors.black,
                    padding: const EdgeInsets.symmetric(
                      horizontal: 8,
                      vertical: 4,
                    ),
                    textStyle: const TextStyle(fontSize: 11),
                    minimumSize: const Size(0, 24),
                  ),
                ),
              const SizedBox(width: 8),
              ElevatedButton.icon(
                onPressed: () =>
                    _openQueryTab(tab.connection, tab.database, tab.schema),
                icon: const Icon(Icons.code, size: 12),
                label: const Text('SQL Editor'),
                style: ElevatedButton.styleFrom(
                  backgroundColor: const Color(0xFF007ACC),
                  foregroundColor: Colors.white,
                  padding: const EdgeInsets.symmetric(
                    horizontal: 6,
                    vertical: 4,
                  ),
                  textStyle: const TextStyle(fontSize: 11),
                  minimumSize: const Size(0, 24),
                ),
              ),
              const SizedBox(width: 8),
              InkWell(
                onTap: () => _loadDatabaseInfo(tab),
                child: const Icon(
                  Icons.refresh,
                  size: 16,
                  color: Colors.white54,
                ),
              ),
            ],
          ),
        ),
        // Content
        Expanded(
          child: Builder(
            builder: (ctx) {
              if (loading)
                return const Center(child: CircularProgressIndicator());
              if (error != null)
                return Center(
                  child: Text(
                    'Error: $error',
                    style: const TextStyle(color: Colors.red),
                  ),
                );
              if (items == null || items.isEmpty)
                return const Center(
                  child: Text(
                    'No items found.',
                    style: TextStyle(color: Colors.white54),
                  ),
                );

              return ListView.builder(
                padding: const EdgeInsets.all(16),
                itemCount: items.length,
                itemBuilder: (ctx, i) {
                  final item = items[i];
                  return Card(
                    color: const Color(0xFF252526),
                    margin: const EdgeInsets.only(bottom: 8),
                    child: ListTile(
                      leading: Icon(
                        canCreateTable
                            ? Icons.table_chart_outlined
                            : Icons.folder_outlined,
                        color: canCreateTable
                            ? const Color(0xFF4EC9B0)
                            : const Color(0xFFE8AB53),
                      ),
                      title: Text(
                        item,
                        style: const TextStyle(color: Colors.white),
                      ),
                      trailing: canCreateTable
                          ? IconButton(
                              icon: const Icon(
                                Icons.arrow_forward_ios,
                                size: 14,
                                color: Colors.white54,
                              ),
                              onPressed: () {
                                final fakeNode = _DbNode(
                                  label: item,
                                  type: _NodeType.table,
                                  connection: tab.connection,
                                  database: tab.database,
                                  schema: tab.schema,
                                  table: item,
                                );
                                _openTableTab(fakeNode);
                              },
                            )
                          : null,
                      onTap: canCreateTable
                          ? () {
                              final fakeNode = _DbNode(
                                label: item,
                                type: _NodeType.table,
                                connection: tab.connection,
                                database: tab.database,
                                schema: tab.schema,
                                table: item,
                              );
                              _openTableTab(fakeNode);
                            }
                          : null,
                    ),
                  );
                },
              );
            },
          ),
        ),
      ],
    );
  }

  // ── Table Data View (with sub-tabs: Data | Structure) ────────────────

  Widget _buildTableDataView(_DbTab tab) {
    final hasSelection = _selectedRowData.containsKey(tab.id);
    return Column(
      children: [
        // Sub-tab row: Data | Structure + CRUD buttons
        Container(
          height: 36,
          color: const Color(0xFF2D2D30),
          padding: const EdgeInsets.symmetric(horizontal: 4),
          child: Row(
            children: [
              _buildSubTab('Data', true, () {}),
              _buildSubTab('Structure', false, () {
                final structTab = _DbTab(
                  id: tab.id.replaceFirst('data:', 'struct:'),
                  title: '${tab.title} (Structure)',
                  type: _DbTabType.tableStructure,
                  connection: tab.connection,
                  database: tab.database,
                  schema: tab.schema,
                  table: tab.table,
                );
                setState(() {
                  _openTabs[_activeTabIndex] = structTab;
                });
                if (_structureCache[tab.id] == null) {
                  _loadTableStructure(structTab);
                }
              }),
              const SizedBox(width: 8),
              Container(width: 1, height: 20, color: const Color(0xFF555555)),
              const SizedBox(width: 4),
              // ── CRUD Toolbar ──
              _crudButton(
                Icons.add,
                'Insert Row',
                const Color(0xFF4EC9B0),
                () => _showInsertRowDialog(tab),
              ),
              _crudButton(
                Icons.edit_outlined,
                'Edit Row',
                const Color(0xFFDCDCAA),
                hasSelection ? () => _showEditRowDialog(tab) : null,
              ),
              _crudButton(
                Icons.delete_outline,
                'Delete Row',
                Colors.red,
                hasSelection ? () => _deleteSelectedRow(tab) : null,
              ),
              const Spacer(),
              // Row count
              if (_dataCache[tab.id] != null)
                Padding(
                  padding: const EdgeInsets.only(right: 4),
                  child: Text(
                    '${_dataCache[tab.id]!.rows.length} rows',
                    style: const TextStyle(color: Colors.white24, fontSize: 11),
                  ),
                ),
              _crudButton(
                Icons.refresh,
                'Refresh',
                Colors.white54,
                () => _loadTableData(tab),
              ),
            ],
          ),
        ),
        // Data grid
        Expanded(child: _buildDataGrid(tab)),
      ],
    );
  }

  Widget _crudButton(
    IconData icon,
    String tooltip,
    Color color,
    VoidCallback? onTap,
  ) {
    return Tooltip(
      message: tooltip,
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(4),
        child: Padding(
          padding: const EdgeInsets.all(6),
          child: Icon(
            icon,
            size: 16,
            color: onTap != null ? color : color.withAlpha(80),
          ),
        ),
      ),
    );
  }

  Widget _buildTableStructureView(_DbTab tab) {
    final originalId = tab.id.replaceFirst('struct:', 'data:');
    return Column(
      children: [
        Container(
          height: 32,
          color: const Color(0xFF2D2D30),
          child: Row(
            children: [
              _buildSubTab('Data', false, () {
                final dataTab = _DbTab(
                  id: originalId,
                  title: tab.title.replaceAll(' (Structure)', ''),
                  type: _DbTabType.tableData,
                  connection: tab.connection,
                  database: tab.database,
                  schema: tab.schema,
                  table: tab.table,
                );
                setState(() {
                  _openTabs[_activeTabIndex] = dataTab;
                });
                if (_dataCache[originalId] == null) {
                  _loadTableData(dataTab);
                }
              }),
              _buildSubTab('Structure', true, () {}),
              const Spacer(),
              Padding(
                padding: const EdgeInsets.only(right: 8),
                child: InkWell(
                  onTap: () => _loadTableStructure(tab),
                  child: const Icon(
                    Icons.refresh,
                    size: 14,
                    color: Colors.white54,
                  ),
                ),
              ),
            ],
          ),
        ),
        Expanded(child: _buildStructureGrid(tab)),
      ],
    );
  }

  Widget _buildSubTab(String label, bool isActive, VoidCallback onTap) {
    return InkWell(
      onTap: onTap,
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 16),
        decoration: BoxDecoration(
          border: Border(
            bottom: BorderSide(
              color: isActive ? const Color(0xFF007ACC) : Colors.transparent,
              width: 2,
            ),
          ),
        ),
        alignment: Alignment.center,
        child: Text(
          label,
          style: TextStyle(
            color: isActive ? Colors.white : Colors.white54,
            fontSize: 12,
            fontWeight: isActive ? FontWeight.w600 : FontWeight.normal,
          ),
        ),
      ),
    );
  }

  Widget _buildDataGrid(_DbTab tab) {
    final loading = _loadingState[tab.id] ?? false;
    final error = _errorState[tab.id];
    final data = _dataCache[tab.id];

    if (loading) return const Center(child: CircularProgressIndicator());
    if (error != null) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Text(
            'Error: $error',
            style: const TextStyle(color: Colors.red),
          ),
        ),
      );
    }
    if (data == null || data.columns.isEmpty) {
      return const Center(
        child: Text('No data', style: TextStyle(color: Colors.white38)),
      );
    }
    final page = _pageState[tab.id] ?? 0;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Expanded(child: _buildPlutoGrid(data.columns, data.rows, tabId: tab.id)),
        Container(
          height: 28,
          padding: const EdgeInsets.symmetric(horizontal: 16),
          decoration: const BoxDecoration(
            color: Color(0xFF252526),
            border: Border(top: BorderSide(color: Color(0xFF3C3C3C))),
          ),
          child: Row(
            mainAxisAlignment: MainAxisAlignment.end,
            children: [
              Text(
                'Page ${page + 1}',
                style: const TextStyle(color: Colors.white70, fontSize: 11),
              ),
              const SizedBox(width: 16),
              InkWell(
                onTap: page > 0
                    ? () {
                        setState(() => _pageState[tab.id] = page - 1);
                        _loadTableData(tab);
                      }
                    : null,
                child: Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 2),
                  child: Icon(
                    Icons.chevron_left,
                    size: 16,
                    color: page > 0 ? Colors.white : Colors.white24,
                  ),
                ),
              ),
              const SizedBox(width: 8),
              InkWell(
                onTap: data.rows.length == 100
                    ? () {
                        setState(() => _pageState[tab.id] = page + 1);
                        _loadTableData(tab);
                      }
                    : null,
                child: Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 2),
                  child: Icon(
                    Icons.chevron_right,
                    size: 16,
                    color: data.rows.length == 100 ? Colors.white : Colors.white24,
                  ),
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }


  String _formatTimeRelative(DateTime time) {
    final now = DateTime.now();
    final diff = now.difference(time);
    if (diff.inHours < 24) {
      if (diff.inSeconds < 60) return '${diff.inSeconds} detik lalu';
      if (diff.inMinutes < 60) return '${diff.inMinutes} menit lalu';
      return '${diff.inHours} jam lalu';
    }
    return '${time.day}/${time.month}/${time.year} ${time.hour}:${time.minute.toString().padLeft(2, '0')}';
  }

  Widget _buildQueryHistoryGrid(_DbTab tab) {
    return FutureBuilder<List<QueryHistoryItem>>(
      future: QueryHistoryService.getHistory(),
      builder: (context, snapshot) {
        if (snapshot.connectionState == ConnectionState.waiting) {
          return const Center(child: CircularProgressIndicator());
        }
        final history = snapshot.data ?? [];
        if (history.isEmpty) {
          return const Center(
            child: Text('Belum ada history query.', style: TextStyle(color: Colors.white38)),
          );
        }

        return Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Padding(
              padding: const EdgeInsets.all(16.0),
              child: Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  const Text('Query History', style: TextStyle(color: Colors.white, fontSize: 18, fontWeight: FontWeight.bold)),
                  TextButton.icon(
                    icon: const Icon(Icons.delete_outline, color: Colors.redAccent),
                    label: const Text('Clear History', style: TextStyle(color: Colors.redAccent)),
                    onPressed: () async {
                      await QueryHistoryService.clearHistory();
                      setState(() {});
                    },
                  ),
                ],
              ),
            ),
            Expanded(
              child: ListView.separated(
                itemCount: history.length,
                separatorBuilder: (context, index) => const Divider(color: Color(0xFF3C3C3C)),
                itemBuilder: (context, index) {
                  final item = history[index];
                  return ListTile(
                    title: Text(item.query, style: const TextStyle(color: Color(0xFF4EC9B0), fontFamily: 'monospace')),
                    subtitle: Padding(
                      padding: const EdgeInsets.only(top: 8.0),
                      child: Text(
                        'Executed: ${_formatTimeRelative(item.executedAt)} | Connection: ${item.connectionName}' + (item.database != null ? ' | DB: ${item.database}' : ''),
                        style: const TextStyle(color: Colors.white54, fontSize: 12),
                      ),
                    ),
                  );
                },
              ),
            ),
          ],
        );
      },
    );
  }

  Widget _buildStructureGrid(_DbTab tab) {
    final originalId = tab.id.replaceFirst('struct:', 'data:');
    final structId = 'struct:${tab.id}';
    final loading = _loadingState[structId] ?? false;
    final error = _errorState[structId];
    final data = _structureCache[originalId] ?? _structureCache[tab.id];

    if (loading) return const Center(child: CircularProgressIndicator());
    if (error != null) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Text(
            'Error: $error',
            style: const TextStyle(color: Colors.red),
          ),
        ),
      );
    }
    if (data == null || data.isEmpty) {
      return const Center(
        child: Text(
          'No structure data',
          style: TextStyle(color: Colors.white38),
        ),
      );
    }
    final cols = data.first.keys.toList();
    return _buildPlutoGrid(cols, data);
  }

  // ── Query View ───────────────────────────────────────────────────────────

  Widget _buildQueryView(_DbTab tab) {
    final ctrl = _queryControllers[tab.id];
    if (ctrl == null) return const SizedBox.shrink();

    final focusNode = _queryFocusNodes.putIfAbsent(tab.id, () => FocusNode());

    final isExecuting = _queryExecuting[tab.id] ?? false;
    final result = _queryResults[tab.id];
    final error = _queryErrors[tab.id];

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        // SQL Editor area
        Container(
          height: 180,
          padding: const EdgeInsets.all(8),
          color: const Color(0xFF1E1E1E),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              // Toolbar
              Row(
                children: [
                  const Icon(Icons.code, size: 14, color: Color(0xFF007ACC)),
                  const SizedBox(width: 6),
                  const Text(
                    'SQL Editor',
                    style: TextStyle(
                      color: Colors.white54,
                      fontSize: 11,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                  const Spacer(),
                  ElevatedButton.icon(
                    onPressed: isExecuting ? null : () => _executeQuery(tab),
                    icon: Icon(
                      isExecuting ? Icons.hourglass_empty : Icons.play_arrow,
                      size: 12,
                    ),
                    label: Text(isExecuting ? 'Running...' : 'Run'),
                    style: ElevatedButton.styleFrom(
                      backgroundColor: const Color(0xFF007ACC),
                      foregroundColor: Colors.white,
                      padding: const EdgeInsets.symmetric(
                        horizontal: 8,
                        vertical: 4,
                      ),
                      textStyle: const TextStyle(fontSize: 11),
                      minimumSize: const Size(0, 24),
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 6),
              // Text field
              Expanded(
                child: Container(
                  decoration: BoxDecoration(
                    color: const Color(0xFF252526),
                    borderRadius: BorderRadius.circular(4),
                    border: Border.all(color: const Color(0xFF3C3C3C)),
                  ),
                  padding: const EdgeInsets.all(8),
                  child: RawAutocomplete<_SqlSuggestion>(
                    textEditingController: ctrl,
                    focusNode: focusNode,
                    optionsBuilder: (TextEditingValue textEditingValue) {
                      final text = textEditingValue.text;
                      final selection = textEditingValue.selection;
                      if (!selection.isValid || selection.isDirectional) {
                        return const Iterable<_SqlSuggestion>.empty();
                      }
                      final offset = selection.baseOffset;

                      int start = offset - 1;
                      while (start >= 0 &&
                          RegExp(r'[a-zA-Z0-9_]').hasMatch(text[start])) {
                        start--;
                      }
                      start++;

                      if (start >= offset)
                        return const Iterable<_SqlSuggestion>.empty();

                      final currentWord = text.substring(start, offset);
                      if (currentWord.isEmpty)
                        return const Iterable<_SqlSuggestion>.empty();

                      final allSuggestions =
                          _querySuggestions[tab.id] ?? <String>[];
                      final matches = allSuggestions
                          .where(
                            (s) => s.toLowerCase().startsWith(
                              currentWord.toLowerCase(),
                            ),
                          )
                          .toList();

                      if (matches.isEmpty)
                        return const Iterable<_SqlSuggestion>.empty();

                      return matches.map((m) {
                        final fullText =
                            text.substring(0, start) +
                            m +
                            text.substring(offset);
                        final newOffset = start + m.length;
                        return _SqlSuggestion(m, fullText, newOffset);
                      });
                    },
                    displayStringForOption: (option) => option.fullText,
                    onSelected: (option) {
                      ctrl.selection = TextSelection.collapsed(
                        offset: option.newOffset,
                      );
                    },
                    optionsViewBuilder: (context, onSelected, options) {
                      return Align(
                        alignment: Alignment.topLeft,
                        child: Material(
                          elevation: 4,
                          color: const Color(0xFF2D2D30),
                          child: ConstrainedBox(
                            constraints: const BoxConstraints(
                              maxHeight: 200,
                              maxWidth: 250,
                            ),
                            child: ListView.builder(
                              padding: EdgeInsets.zero,
                              shrinkWrap: true,
                              itemCount: options.length,
                              itemBuilder: (context, index) {
                                final option = options.elementAt(index);
                                return InkWell(
                                  onTap: () => onSelected(option),
                                  child: Padding(
                                    padding: const EdgeInsets.symmetric(
                                      horizontal: 12,
                                      vertical: 8,
                                    ),
                                    child: Text(
                                      option.label,
                                      style: const TextStyle(
                                        color: Colors.white,
                                        fontFamily: 'monospace',
                                        fontSize: 13,
                                      ),
                                    ),
                                  ),
                                );
                              },
                            ),
                          ),
                        ),
                      );
                    },
                    fieldViewBuilder:
                        (context, controller, focusNode, onFieldSubmitted) {
                          final customTheme = Map<String, TextStyle>.from(
                            vs2015Theme,
                          );
                          customTheme['root'] =
                              customTheme['root']?.copyWith(
                                backgroundColor: Colors.transparent,
                              ) ??
                              const TextStyle(
                                backgroundColor: Colors.transparent,
                              );

                          return CodeTheme(
                            data: CodeThemeData(styles: customTheme),
                            child: Theme(
                              data: Theme.of(context).copyWith(
                                inputDecorationTheme:
                                    const InputDecorationTheme(
                                      border: InputBorder.none,
                                      filled: false,
                                      isDense: true,
                                      contentPadding: EdgeInsets.zero,
                                    ),
                              ),
                              child: TextField(
                                controller: controller as CodeController,
                                focusNode: focusNode,
                                maxLines: null,
                                style: const TextStyle(
                                  color: Colors.white,
                                  fontFamily: 'monospace',
                                  fontSize: 13,
                                  height: 1.6,
                                ),
                                decoration: const InputDecoration(
                                  border: InputBorder.none,
                                  focusedBorder: InputBorder.none,
                                  enabledBorder: InputBorder.none,
                                  errorBorder: InputBorder.none,
                                  disabledBorder: InputBorder.none,
                                  filled: true,
                                  fillColor: Colors.transparent,
                                  isDense: true,
                                  contentPadding: EdgeInsets.zero,
                                ),
                              ),
                            ),
                          );
                        },
                  ),
                ),
              ),
            ],
          ),
        ),
        // Divider
        Container(height: 1, color: const Color(0xFF007ACC)),
        // Results area
        Expanded(
          child: Container(
            color: const Color(0xFF1E1E1E),
            child: _buildQueryResults(isExecuting, result, error),
          ),
        ),
      ],
    );
  }

  Widget _buildQueryResults(
    bool isExecuting,
    QueryResult? result,
    String? error,
  ) {
    if (isExecuting) {
      return const Center(child: CircularProgressIndicator());
    }
    if (error != null) {
      return SingleChildScrollView(
        padding: const EdgeInsets.all(16),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Icon(Icons.error_outline, color: Colors.red, size: 16),
            const SizedBox(width: 8),
            Expanded(
              child: Text(
                error,
                style: const TextStyle(color: Colors.red, fontSize: 13),
              ),
            ),
          ],
        ),
      );
    }
    if (result == null) {
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: const [
            Icon(Icons.terminal, size: 32, color: Colors.white12),
            SizedBox(height: 8),
            Text(
              'Write a query and press Run',
              style: TextStyle(color: Colors.white24, fontSize: 13),
            ),
          ],
        ),
      );
    }
    if (result.columns.isEmpty) {
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(
              Icons.check_circle_outline,
              size: 32,
              color: Colors.green,
            ),
            const SizedBox(height: 8),
            Text(
              'Query executed successfully\nAffected rows: ${result.affectedRows}',
              textAlign: TextAlign.center,
              style: const TextStyle(color: Colors.green, fontSize: 13),
            ),
          ],
        ),
      );
    }
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // Result header
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
          color: const Color(0xFF252526),
          child: Row(
            children: [
              const Icon(Icons.table_chart, size: 13, color: Colors.white38),
              const SizedBox(width: 6),
              Text(
                '${result.rows.length} rows × ${result.columns.length} columns',
                style: const TextStyle(color: Colors.white38, fontSize: 11),
              ),
            ],
          ),
        ),
        Expanded(child: _buildPlutoGrid(result.columns, result.rows)),
      ],
    );
  }

  // ── PlutoGrid ────────────────────────────────────────────────────────────

  Widget _buildPlutoGrid(
    List<String> cols,
    List<Map<String, dynamic>> rowsData, {
    String? tabId,
  }) {
    if (cols.isEmpty) return const SizedBox.shrink();

    final columns = cols.map((c) {
      return PlutoColumn(
        title: c,
        field: c,
        type: PlutoColumnType.text(),
        readOnly: true,
        enableEditingMode: false,
      );
    }).toList();

    final rows = rowsData.asMap().entries.map((entry) {
      final r = entry.value;
      final cells = <String, PlutoCell>{};
      for (final c in cols) {
        cells[c] = PlutoCell(value: r[c]?.toString() ?? 'NULL');
      }
      return PlutoRow(cells: cells);
    }).toList();

    return PlutoGrid(
      key: ValueKey(tabId ?? 'struct_${cols.hashCode}'),
      columns: columns,
      rows: rows,
      mode: PlutoGridMode.selectWithOneTap,
      onSelected: tabId != null
          ? (PlutoGridOnSelectedEvent event) {
              final rowIdx = event.rowIdx;
              if (rowIdx != null && rowIdx >= 0 && rowIdx < rowsData.length) {
                setState(() {
                  _selectedRowIndex[tabId] = rowIdx;
                  _selectedRowData[tabId] = Map<String, dynamic>.from(
                    rowsData[rowIdx],
                  );
                });
              }
            }
          : null,
      configuration: const PlutoGridConfiguration(
        style: PlutoGridStyleConfig(
          gridBackgroundColor: Color(0xFF1E1E1E),
          rowColor: Color(0xFF1E1E1E),
          menuBackgroundColor: Color(0xFF2D2D30),
          gridBorderColor: Color(0xFF333333),
          borderColor: Color(0xFF333333),
          activatedBorderColor: Color(0xFF007ACC),
          activatedColor: Color(0xFF094771),
          cellTextStyle: TextStyle(color: Colors.white70, fontSize: 13),
          columnTextStyle: TextStyle(
            color: Colors.white,
            fontWeight: FontWeight.bold,
            fontSize: 13,
          ),
          iconColor: Colors.white54,
        ),
      ),
    );
  }
}

class _SqlSuggestion {
  final String label;
  final String fullText;
  final int newOffset;
  _SqlSuggestion(this.label, this.fullText, this.newOffset);

  @override
  String toString() => fullText;
}
