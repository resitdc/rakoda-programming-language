import re

with open("lib/features/database/database_workspace.dart", "r") as f:
    content = f.read()

# 1. Add _pageState
state_var = "  final Map<String, int> _pageState = {};"
if state_var not in content:
    content = content.replace("  final Map<String, String?> _errorState = {};", "  final Map<String, String?> _errorState = {};\n" + state_var)

# 2. Update _loadTableData
old_load = """      String q;
      if (tab.connection.engine == DatabaseEngine.postgres) {
        q = 'SELECT * FROM "${tab.schema}"."${tab.table}" LIMIT 1000';
      } else if (tab.connection.engine == DatabaseEngine.mysql) {
        q = 'SELECT * FROM `${tab.database}`.`${tab.table}` LIMIT 1000';
      } else {
        q = 'SELECT * FROM "${tab.table}" LIMIT 1000';
      }"""

new_load = """      final page = _pageState[tab.id] ?? 0;
      final offset = page * 100;
      String q;
      if (tab.connection.engine == DatabaseEngine.postgres) {
        q = 'SELECT * FROM "${tab.schema}"."${tab.table}" LIMIT 100 OFFSET $offset';
      } else if (tab.connection.engine == DatabaseEngine.mysql) {
        q = 'SELECT * FROM `${tab.database}`.`${tab.table}` LIMIT 100 OFFSET $offset';
      } else {
        q = 'SELECT * FROM "${tab.table}" LIMIT 100 OFFSET $offset';
      }"""

content = content.replace(old_load, new_load)

# 3. Update _buildDataGrid
old_grid = "    return _buildPlutoGrid(data.columns, data.rows, tabId: tab.id);"

new_grid = """    final page = _pageState[tab.id] ?? 0;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Expanded(child: _buildPlutoGrid(data.columns, data.rows, tabId: tab.id)),
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          decoration: const BoxDecoration(
            color: Color(0xFF252526),
            border: Border(top: BorderSide(color: Color(0xFF3C3C3C))),
          ),
          child: Row(
            mainAxisAlignment: MainAxisAlignment.end,
            children: [
              Text(
                'Page ${page + 1}',
                style: const TextStyle(color: Colors.white70, fontSize: 13),
              ),
              const SizedBox(width: 16),
              IconButton(
                icon: const Icon(Icons.chevron_left, size: 20),
                color: Colors.white,
                onPressed: page > 0
                    ? () {
                        setState(() => _pageState[tab.id] = page - 1);
                        _loadTableData(tab);
                      }
                    : null,
                tooltip: 'Previous Page',
              ),
              IconButton(
                icon: const Icon(Icons.chevron_right, size: 20),
                color: Colors.white,
                onPressed: data.rows.length == 100
                    ? () {
                        setState(() => _pageState[tab.id] = page + 1);
                        _loadTableData(tab);
                      }
                    : null,
                tooltip: 'Next Page',
              ),
            ],
          ),
        ),
      ],
    );"""

content = content.replace(old_grid, new_grid)

with open("lib/features/database/database_workspace.dart", "w") as f:
    f.write(content)
