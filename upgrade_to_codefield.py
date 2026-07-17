import re

with open("apps/studio/lib/features/database/database_workspace.dart", "r") as f:
    content = f.read()

# 1. Add imports
imports = """
import 'package:flutter_code_editor/flutter_code_editor.dart';
import 'package:flutter_highlight/themes/vs2015.dart';
import 'package:highlight/languages/sql.dart';
"""
if "highlight/languages/sql.dart" not in content:
    content = content.replace("import 'package:flutter/material.dart';", "import 'package:flutter/material.dart';\n" + imports.strip('\n'))

# 2. Change controller initialization
content = content.replace(
    "_queryControllers.putIfAbsent(tabId, () => TextEditingController());",
    "_queryControllers.putIfAbsent(tabId, () => CodeController(language: sql));"
)

# 3. Replace TextField with CodeField
old_textfield = """                    fieldViewBuilder: (context, controller, focusNode, onFieldSubmitted) {
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
                    },"""

new_codefield = """                    fieldViewBuilder: (context, controller, focusNode, onFieldSubmitted) {
                      final customTheme = Map<String, TextStyle>.from(vs2015Theme);
                      customTheme['root'] = customTheme['root']?.copyWith(
                        backgroundColor: Colors.transparent,
                      ) ?? const TextStyle(backgroundColor: Colors.transparent);
                      
                      return CodeTheme(
                        data: CodeThemeData(styles: customTheme),
                        child: Theme(
                          data: Theme.of(context).copyWith(
                            inputDecorationTheme: const InputDecorationTheme(
                              border: InputBorder.none,
                              filled: false,
                            ),
                          ),
                          child: CodeField(
                            controller: controller as CodeController,
                            focusNode: focusNode,
                            textStyle: const TextStyle(
                              fontFamily: 'monospace',
                              fontSize: 13,
                              height: 1.6,
                            ),
                            background: Colors.transparent,
                            gutterStyle: const GutterStyle(showLineNumbers: false, showErrors: false, showFoldingHandles: false, margin: 0),
                          ),
                        ),
                      );
                    },"""

if old_textfield in content:
    content = content.replace(old_textfield, new_codefield)
else:
    print("Could not find TextField in fieldViewBuilder!")

with open("apps/studio/lib/features/database/database_workspace.dart", "w") as f:
    f.write(content)

