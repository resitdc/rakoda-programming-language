import re

with open('lib/features/database/database_workspace.dart', 'r') as f:
    content = f.read()

# 1. Add _DbTabType.queryHistory
if 'queryHistory' not in content:
    content = content.replace(
        'enum _DbTabType { tableData, tableStructure, query, databaseInfo }',
        'enum _DbTabType { tableData, tableStructure, query, databaseInfo, queryHistory }'
    )

# 2. Add import for query_history_service.dart
if 'query_history_service.dart' not in content:
    content = content.replace(
        "import 'package:flutter/material.dart';",
        "import 'package:flutter/material.dart';\nimport 'query_history_service.dart';"
    )

# 3. Add _openQueryHistoryTab method
open_history_method = """  void _openQueryHistoryTab() {
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
  }"""

if '_openQueryHistoryTab' not in content:
    content = content.replace(
        "  void _openQueryTab(DatabaseConnection conn, String? db, String? schema) {",
        open_history_method + "\n\n  void _openQueryTab(DatabaseConnection conn, String? db, String? schema) {"
    )

# 4. Add menu item in _showNodeMenu
menu_item = """
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
"""
if 'Query History' not in content:
    content = content.replace(
        "    // All nodes: SQL Editor",
        menu_item + "\n    // All nodes: SQL Editor"
    )

# 5. Add UI logic in _buildTabContent
history_ui = """      case _DbTabType.queryHistory:
        return _buildQueryHistoryGrid(tab);"""

if '_buildQueryHistoryGrid' not in content:
    content = content.replace(
        "      case _DbTabType.databaseInfo:\n        return _buildDatabaseInfo(tab);",
        "      case _DbTabType.databaseInfo:\n        return _buildDatabaseInfo(tab);\n" + history_ui
    )


# 6. Add _buildQueryHistoryGrid implementation
time_formatter = """
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
"""

history_grid = time_formatter + """
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
"""

if '_buildQueryHistoryGrid' not in content:
    content = content.replace(
        "  Widget _buildStructureGrid(_DbTab tab) {",
        history_grid + "\n  Widget _buildStructureGrid(_DbTab tab) {"
    )

# 7. Inject history tracking in _executeQuery
tracker_logic = """
      QueryHistoryService.addHistory(QueryHistoryItem(
        query: query,
        executedAt: DateTime.now(),
        connectionName: tab.connection.name,
        database: tab.database,
      ));
"""
# There are two places executeQuery is called for the actual query.
# Let's use a regex to insert after `final res = await svc.executeQuery(query);`
if 'QueryHistoryService.addHistory' not in content:
    content = content.replace(
        "final res = await svc.executeQuery(query);",
        "final res = await svc.executeQuery(query);\n" + tracker_logic
    )


with open('lib/features/database/database_workspace.dart', 'w') as f:
    f.write(content)

