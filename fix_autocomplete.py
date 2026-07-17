import re

with open("apps/studio/lib/features/database/database_workspace.dart", "r") as f:
    content = f.read()

# 1. Add _SqlSuggestion class at the end of the file
suggestion_class = """
class _SqlSuggestion {
  final String label;
  final String fullText;
  final int newOffset;
  _SqlSuggestion(this.label, this.fullText, this.newOffset);

  @override
  String toString() => fullText;
}
"""
if "_SqlSuggestion" not in content:
    content += "\n" + suggestion_class

# 2. Add state variables
state_vars = """
  // ── Query tab state ──
  final Map<String, TextEditingController> _queryControllers = {};
  final Map<String, FocusNode> _queryFocusNodes = {};
  final Map<String, List<String>> _querySuggestions = {};
"""
content = re.sub(
    r"  // ── Query tab state ──\n  final Map<String, TextEditingController> _queryControllers = {};",
    state_vars.strip('\n'),
    content
)

# 3. Dispose focus nodes
dispose_code = """
    for (final c in _queryControllers.values) {
      c.dispose();
    }
    for (final fn in _queryFocusNodes.values) {
      fn.dispose();
    }
"""
content = re.sub(
    r"    for \(final c in _queryControllers\.values\) \{\n      c\.dispose\(\);\n    \}",
    dispose_code.strip('\n'),
    content
)

# 4. _openQueryTab logic (focus node init and fetch)
fetch_method = """
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
            final colName = colDef['name']?.toString() ?? colDef['Field']?.toString() ?? colDef['column_name']?.toString() ?? '';
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
"""

if "_fetchSuggestionsForQueryTab" not in content:
    content = re.sub(
        r"(  void _openQueryTab\(DatabaseConnection conn, String\? db, String\? schema\) \{)",
        fetch_method + r"\n\1",
        content
    )

init_focus = """
    _queryControllers.putIfAbsent(tabId, () => TextEditingController());
    _queryFocusNodes.putIfAbsent(tabId, () => FocusNode());
"""
content = re.sub(
    r"    _queryControllers\.putIfAbsent\(tabId, \(\) => TextEditingController\(\)\);",
    init_focus.strip('\n'),
    content
)

fetch_call = """
    setState(() {
      _openTabs.add(tab);
      _activeTabIndex = _openTabs.length - 1;
    });
    _fetchSuggestionsForQueryTab(tab);
"""
content = re.sub(
    r"    setState\(\(\) \{\n      _openTabs\.add\(tab\);\n      _activeTabIndex = _openTabs\.length - 1;\n    \}\);",
    fetch_call.strip('\n'),
    content
)

# 5. Replace TextField with RawAutocomplete
# We look for `child: TextField(\n                    controller: ctrl,\n                    maxLines: null,`
old_textfield = """                  child: TextField(
                    controller: ctrl,
                    maxLines: null,
                    style: const TextStyle(
                      color: Colors.white,
                      fontFamily: 'monospace',
                      fontSize: 13,
                      height: 1.5,
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
                      hintText:
                          'SELECT * FROM table_name;\\n\\nCREATE TABLE users (\\n  id INTEGER PRIMARY KEY,\\n  name TEXT NOT NULL\\n);',
                      hintStyle: TextStyle(color: Colors.white24),
                    ),
                  ),"""

new_autocomplete = """                  child: RawAutocomplete<_SqlSuggestion>(
                    textEditingController: ctrl,
                    focusNode: _queryFocusNodes[tab.id]!,
                    optionsBuilder: (TextEditingValue textEditingValue) {
                      final text = textEditingValue.text;
                      final selection = textEditingValue.selection;
                      if (!selection.isValid || selection.isDirectional) {
                        return const Iterable<_SqlSuggestion>.empty();
                      }
                      final offset = selection.baseOffset;
                      
                      int start = offset - 1;
                      while (start >= 0 && RegExp(r'[a-zA-Z0-9_]').hasMatch(text[start])) {
                        start--;
                      }
                      start++;
                      
                      if (start >= offset) return const Iterable<_SqlSuggestion>.empty();
                      
                      final currentWord = text.substring(start, offset);
                      if (currentWord.isEmpty) return const Iterable<_SqlSuggestion>.empty();
                      
                      final allSuggestions = _querySuggestions[tab.id] ?? <String>[];
                      final matches = allSuggestions
                          .where((s) => s.toLowerCase().startsWith(currentWord.toLowerCase()))
                          .toList();
                          
                      if (matches.isEmpty) return const Iterable<_SqlSuggestion>.empty();
                      
                      return matches.map((m) {
                        final fullText = text.substring(0, start) + m + text.substring(offset);
                        final newOffset = start + m.length;
                        return _SqlSuggestion(m, fullText, newOffset);
                      });
                    },
                    displayStringForOption: (option) => option.fullText,
                    onSelected: (option) {
                      ctrl.selection = TextSelection.collapsed(offset: option.newOffset);
                    },
                    optionsViewBuilder: (context, onSelected, options) {
                      return Align(
                        alignment: Alignment.topLeft,
                        child: Material(
                          elevation: 4,
                          color: const Color(0xFF2D2D30),
                          child: ConstrainedBox(
                            constraints: const BoxConstraints(maxHeight: 200, maxWidth: 250),
                            child: ListView.builder(
                              padding: EdgeInsets.zero,
                              shrinkWrap: true,
                              itemCount: options.length,
                              itemBuilder: (context, index) {
                                final option = options.elementAt(index);
                                return InkWell(
                                  onTap: () => onSelected(option),
                                  child: Padding(
                                    padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                                    child: Text(
                                      option.label,
                                      style: const TextStyle(color: Colors.white, fontFamily: 'monospace', fontSize: 13),
                                    ),
                                  ),
                                );
                              },
                            ),
                          ),
                        ),
                      );
                    },
                    fieldViewBuilder: (context, controller, focusNode, onFieldSubmitted) {
                      return TextField(
                        controller: controller,
                        focusNode: focusNode,
                        maxLines: null,
                        style: const TextStyle(
                          color: Colors.white,
                          fontFamily: 'monospace',
                          fontSize: 13,
                          height: 1.5,
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
                          hintText:
                              'SELECT * FROM table_name;\\n\\nCREATE TABLE users (\\n  id INTEGER PRIMARY KEY,\\n  name TEXT NOT NULL\\n);',
                          hintStyle: TextStyle(color: Colors.white24),
                        ),
                      );
                    },
                  ),"""

if old_textfield in content:
    content = content.replace(old_textfield, new_autocomplete)
else:
    print("Could not find TextField to replace!")

with open("apps/studio/lib/features/database/database_workspace.dart", "w") as f:
    f.write(content)

