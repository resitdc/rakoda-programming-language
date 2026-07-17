import re

with open("/Users/resitdc/.gemini/antigravity-ide/brain/89ce15c4-9392-4416-91d8-284ad2146ca1/scratch/backup_db_workspace.dart", "r") as f:
    content = f.read()

# Fix the specific corrupted section around _showNodeMenu
bad_section = """
  void _showNodeMenu(_DbNode node) {
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
"""

good_section = """
  void _showNodeMenu(_DbNode node) {
    final items = <Widget>[];

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
"""

content = content.replace(bad_section.strip(), good_section.strip())

with open("/Users/resitdc/Documents/projects/resitdc/indonesia-programming-language/apps/studio/lib/features/database/database_workspace.dart", "w") as f:
    f.write(content)
