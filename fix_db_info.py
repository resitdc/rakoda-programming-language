import re

with open("/Users/resitdc/Documents/projects/resitdc/indonesia-programming-language/apps/studio/lib/features/database/database_workspace.dart", "r") as f:
    content = f.read()

# Add _openDatabaseInfoTab below _openQueryTab
new_methods = """
  void _openDatabaseInfoTab(_DbNode node) {
    final tabId = 'info:${node.connection.id}:${node.database ?? ''}:${node.schema ?? ''}';
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
    _loadDatabaseInfo(tab);
    _closeSidebarOnMobile();
  }
"""
content = re.sub(
    r"(  void _closeTab\(int index\) {)",
    new_methods + r"\n\1",
    content
)

new_load = """
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
"""

content = re.sub(
    r"(  Future<void> _executeQuery\()",
    new_load + r"\n\1",
    content
)

with open("/Users/resitdc/Documents/projects/resitdc/indonesia-programming-language/apps/studio/lib/features/database/database_workspace.dart", "w") as f:
    f.write(content)
