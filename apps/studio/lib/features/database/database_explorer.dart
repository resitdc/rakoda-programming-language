import 'package:flutter/material.dart';
import '../../models/database_connection.dart';
import '../../services/database/connection_service.dart';
import '../../services/database/database_service.dart';
import 'connection_dialog.dart';

class DatabaseExplorer extends StatefulWidget {
  final String projectPath;
  final Function(DatabaseConnection, String?, String?, String) onTableTap;

  const DatabaseExplorer({super.key, required this.projectPath, required this.onTableTap});

  @override
  State<DatabaseExplorer> createState() => _DatabaseExplorerState();
}

class DbNode {
  final String label;
  final String? database;
  final String? schema;
  final String? table;
  final DatabaseConnection connection;
  final bool isExpandable;
  bool isExpanded;
  List<DbNode>? children;

  DbNode({
    required this.label,
    required this.connection,
    this.database,
    this.schema,
    this.table,
    this.isExpandable = true,
    this.isExpanded = false,
    this.children,
  });
}

class _DatabaseExplorerState extends State<DatabaseExplorer> {
  List<DbNode> _roots = [];
  bool _isLoading = true;

  @override
  void initState() {
    super.initState();
    _loadConnections();
  }

  Future<void> _loadConnections() async {
    setState(() => _isLoading = true);
    final conns = await ConnectionService.getConnections();
    setState(() {
      _roots = conns
          .map((c) => DbNode(label: c.name, connection: c, isExpandable: true))
          .toList();
      _isLoading = false;
    });
  }

  Future<void> _addConnection() async {
    final result = await showDialog<DatabaseConnection>(
      context: context,
      builder: (context) => ConnectionDialog(projectPath: widget.projectPath),
    );
    if (result != null) {
      await ConnectionService.saveConnection(result);
      _loadConnections();
    }
  }

  Future<void> _editConnection(DbNode node) async {
    final result = await showDialog<DatabaseConnection>(
      context: context,
      builder: (context) => ConnectionDialog(projectPath: widget.projectPath, connection: node.connection),
    );
    if (result != null) {
      await ConnectionService.saveConnection(result);
      _loadConnections();
    }
  }

  Future<void> _deleteConnection(DbNode node) async {
    final confirm = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        backgroundColor: const Color(0xFF252526),
        title: const Text(
          'Delete Connection',
          style: TextStyle(color: Colors.white),
        ),
        content: Text(
          'Are you sure you want to delete ${node.connection.name}?',
          style: const TextStyle(color: Colors.white70),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () => Navigator.pop(context, true),
            child: const Text('Delete', style: TextStyle(color: Colors.red)),
          ),
        ],
      ),
    );
    if (confirm == true) {
      await ConnectionService.deleteConnection(node.connection.id);
      _loadConnections();
    }
  }

  Future<void> _toggleExpand(DbNode node) async {
    if (!node.isExpandable) {
      widget.onTableTap(
        node.connection,
        node.database,
        node.schema,
        node.table!,
      );
      return;
    }

    setState(() => node.isExpanded = !node.isExpanded);
    if (node.isExpanded && node.children == null) {
      await _loadChildren(node);
    }
  }

  Future<void> _loadChildren(DbNode node) async {
    try {
      final service = DatabaseService.fromConnection(node.connection);
      await service.connect();

      List<DbNode> newChildren = [];

      if (node.database == null && node.schema == null && node.table == null) {
        // Root connection -> Load Databases
        final dbs = await service.getDatabases();
        newChildren = dbs
            .map(
              (db) =>
                  DbNode(label: db, connection: node.connection, database: db),
            )
            .toList();
      } else if (node.database != null &&
          node.schema == null &&
          node.table == null) {
        // Database -> Load Schemas (or Tables if MySQL/SQLite)
        if (node.connection.engine == DatabaseEngine.postgres) {
          final schemas = await service.getSchemas(node.database!);
          newChildren = schemas
              .map(
                (sch) => DbNode(
                  label: sch,
                  connection: node.connection,
                  database: node.database,
                  schema: sch,
                ),
              )
              .toList();
        } else {
          final tables = await service.getTables(node.database!, '');
          newChildren = tables
              .map(
                (t) => DbNode(
                  label: t,
                  connection: node.connection,
                  database: node.database,
                  schema: '',
                  table: t,
                  isExpandable: false,
                ),
              )
              .toList();
        }
      } else if (node.database != null &&
          node.schema != null &&
          node.table == null) {
        // Schema -> Load Tables
        final tables = await service.getTables(node.database!, node.schema!);
        newChildren = tables
            .map(
              (t) => DbNode(
                label: t,
                connection: node.connection,
                database: node.database,
                schema: node.schema,
                table: t,
                isExpandable: false,
              ),
            )
            .toList();
      }

      setState(() => node.children = newChildren);
      await service.disconnect();
    } catch (e) {
      debugPrint('Error loading children: $e');
      if (mounted) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text('Error: $e')));
        setState(() => node.isExpanded = false);
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      color: const Color(0xFF252526),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
            color: const Color(0xFF2D2D30),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                const Text(
                  'DATABASES',
                  style: TextStyle(
                    color: Colors.white,
                    fontSize: 11,
                    fontWeight: FontWeight.bold,
                    letterSpacing: 0.5,
                  ),
                ),
                Row(
                  children: [
                    InkWell(
                      onTap: _addConnection,
                      child: const Icon(
                        Icons.add,
                        size: 16,
                        color: Colors.white,
                      ),
                    ),
                    const SizedBox(width: 8),
                    InkWell(
                      onTap: _loadConnections,
                      child: const Icon(
                        Icons.refresh,
                        size: 16,
                        color: Colors.white,
                      ),
                    ),
                  ],
                ),
              ],
            ),
          ),
          if (_isLoading)
            const Expanded(child: Center(child: CircularProgressIndicator()))
          else
            Expanded(
              child: ListView(
                padding: const EdgeInsets.symmetric(vertical: 2),
                children: _roots.map((node) => _buildNode(node, 0)).toList(),
              ),
            ),
        ],
      ),
    );
  }

  Widget _buildNode(DbNode node, int depth) {
    final isRoot =
        node.database == null && node.schema == null && node.table == null;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        InkWell(
          onTap: () => _toggleExpand(node),
          onLongPress: isRoot
              ? () {
                  showModalBottomSheet(
                    context: context,
                    backgroundColor: const Color(0xFF2D2D30),
                    builder: (context) => Column(
                      mainAxisSize: MainAxisSize.min,
                      children: [
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
                      ],
                    ),
                  );
                }
              : null,
          hoverColor: const Color(0xFF2A2D2E),
          child: Container(
            padding: EdgeInsets.only(
              left: 8.0 + depth * 14.0,
              right: 4,
              top: 4,
              bottom: 4,
            ),
            child: Row(
              children: [
                if (node.isExpandable)
                  Icon(
                    node.isExpanded ? Icons.expand_more : Icons.chevron_right,
                    size: 14,
                    color: Colors.white38,
                  )
                else
                  const SizedBox(width: 14),
                const SizedBox(width: 4),
                Icon(
                  isRoot
                      ? Icons.dns
                      : (node.isExpandable ? Icons.folder : Icons.table_chart),
                  size: 14,
                  color: isRoot
                      ? const Color(0xFF007ACC)
                      : (node.isExpandable
                            ? const Color(0xFFDCDCAA)
                            : const Color(0xFF4EC9B0)),
                ),
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
              ],
            ),
          ),
        ),
        if (node.isExpanded && node.children != null)
          ...node.children!.map((child) => _buildNode(child, depth + 1)),
      ],
    );
  }
}
