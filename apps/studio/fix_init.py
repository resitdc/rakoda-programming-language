import re

with open("apps/studio/lib/services/database/sqlite_service.dart", "r") as f:
    content = f.read()

# Make sure sqfliteFfiInit is only called once globally
if "bool _ffiInitialized = false;" not in content:
    content = content.replace("class SqliteService extends DatabaseService {", "bool _ffiInitialized = false;\n\nclass SqliteService extends DatabaseService {")
    content = content.replace("sqfliteFfiInit();", "if (!_ffiInitialized) {\n      sqfliteFfiInit();\n      _ffiInitialized = true;\n    }")

with open("apps/studio/lib/services/database/sqlite_service.dart", "w") as f:
    f.write(content)
